use bevy::{
    asset::embedded_asset,
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        prepass::ViewPrepassTextures,
        FullscreenShader,
    },
    ecs::query::QueryItem,
    platform::collections::HashMap,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                texture_2d, texture_2d_multisampled, texture_depth_2d,
                texture_depth_2d_multisampled, uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewTarget},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};

#[derive(Debug, Default)]
pub struct ShowPrepassPlugin;

impl Plugin for ShowPrepassPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "show_prepass.wgsl");

        app.add_plugins(ExtractComponentPlugin::<ShowPrepass>::default());
        app.add_plugins(ExtractComponentPlugin::<ShowPrepassDepthPower>::default());

        app.add_plugins(UniformComponentPlugin::<ShowPrepassUniform>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(RenderStartup, init_pipeline);

        render_app
            .init_resource::<SpecializedRenderPipelines<ShowPrepassPipeline>>()
            .add_render_graph_node::<ViewNodeRunner<ShowPrepassNode>>(Core3d, ShowPrepassLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::Tonemapping,
                    ShowPrepassLabel,
                    Node3d::EndMainPassPostProcessing,
                ),
            )
            .add_systems(
                Render,
                (
                    prepare_uniforms
                        .in_set(RenderSystems::Prepare)
                        .before(RenderSystems::PrepareResources),
                    prepare_pipelines.in_set(RenderSystems::Prepare),
                    prepare_bind_groups.in_set(RenderSystems::PrepareBindGroups),
                ),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, ExtractComponent)]
pub enum ShowPrepass {
    Depth,
    Normals,
    MotionVectors,
}

#[derive(Debug, Clone, Copy, PartialEq, Component, ExtractComponent)]
pub struct ShowPrepassDepthPower(pub f32);

#[derive(Component, Clone, ShaderType)]
struct ShowPrepassUniform {
    depth_power: f32,
    delta_time: f32,
}

fn prepare_uniforms(
    mut commands: Commands,
    views: Query<(Entity, Option<&ShowPrepassDepthPower>), With<ShowPrepass>>,
    time: Res<Time>,
) {
    for (view_entity, depth_power) in &views {
        commands.entity(view_entity).insert(ShowPrepassUniform {
            depth_power: depth_power.map_or(1.0, |d| d.0),
            delta_time: time.delta_secs(),
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShowPrepassPipelineKey {
    show_prepass: ShowPrepass,
    hdr: bool,
    multisampled: bool,
}

impl ShowPrepassPipelineKey {
    fn layout_key(&self) -> ShowPrepassPipelineLayoutKey {
        ShowPrepassPipelineLayoutKey {
            show_prepass: self.show_prepass,
            multisampled: self.multisampled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ShowPrepassPipelineLayoutKey {
    show_prepass: ShowPrepass,
    multisampled: bool,
}

impl ShowPrepassPipelineLayoutKey {
    fn keys() -> impl Iterator<Item = Self> {
        [
            ShowPrepass::Depth,
            ShowPrepass::Normals,
            ShowPrepass::MotionVectors,
        ]
        .into_iter()
        .flat_map(|show_prepass| {
            [true, false].into_iter().map(move |multisampled| Self {
                show_prepass,
                multisampled,
            })
        })
    }
}

#[derive(Resource)]
struct ShowPrepassPipeline {
    shader: Handle<Shader>,
    fullscreen_shader: FullscreenShader,
    layouts: HashMap<ShowPrepassPipelineLayoutKey, BindGroupLayout>,
}

impl SpecializedRenderPipeline for ShowPrepassPipeline {
    type Key = ShowPrepassPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let layout = self.layouts.get(&key.layout_key()).unwrap();

        RenderPipelineDescriptor {
            label: Some("show prepass pipeline".into()),
            layout: vec![layout.clone()],
            vertex: self.fullscreen_shader.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: {
                    let mut defs = match key.show_prepass {
                        ShowPrepass::Depth => vec!["SHOW_DEPTH".into()],
                        ShowPrepass::Normals => vec!["SHOW_NORMALS".into()],
                        ShowPrepass::MotionVectors => vec!["SHOW_MOTION_VECTORS".into()],
                    };
                    if key.multisampled {
                        defs.push("MULTISAMPLED".into());
                    }
                    defs
                },
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: if key.hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: default(),
            depth_stencil: None,
            multisample: default(),
            push_constant_ranges: vec![],
            zero_initialize_workgroup_memory: false,
        }
    }
}

fn init_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    fullscreen_shader: Res<FullscreenShader>,
) {
    let layouts = ShowPrepassPipelineLayoutKey::keys()
        .map(|key| {
            let uniform = uniform_buffer::<ShowPrepassUniform>(true);
            let texture = match key.show_prepass {
                ShowPrepass::Depth => match key.multisampled {
                    true => texture_depth_2d_multisampled(),
                    false => texture_depth_2d(),
                },
                ShowPrepass::Normals => match key.multisampled {
                    true => texture_2d_multisampled(TextureSampleType::Float { filterable: false }),
                    false => texture_2d(TextureSampleType::Float { filterable: false }),
                },
                ShowPrepass::MotionVectors => match key.multisampled {
                    true => texture_2d_multisampled(TextureSampleType::Float { filterable: false }),
                    false => texture_2d(TextureSampleType::Float { filterable: false }),
                },
            };

            let layout = render_device.create_bind_group_layout(
                None,
                &[
                    uniform.build(0, ShaderStages::FRAGMENT),
                    texture.build(1, ShaderStages::FRAGMENT),
                ],
            );
            (key, layout)
        })
        .collect();

    commands.insert_resource(ShowPrepassPipeline {
        shader: asset_server.load("embedded://bevy_show_prepass/show_prepass.wgsl"),
        fullscreen_shader: fullscreen_shader.clone(),
        layouts,
    });
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct ShowPrepassLabel;

#[derive(Debug, Default)]
struct ShowPrepassNode;

impl ViewNode for ShowPrepassNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ShowPrepass,
        &'static ExtractedCamera,
        &'static CachedShowPrepassPipeline,
        &'static ShowPrepassBindGroup,
        &'static DynamicUniformIndex<ShowPrepassUniform>,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, _show_prepass, camera, pipeline, bind_group, uniform_index): QueryItem<
            Self::ViewQuery,
        >,
        world: &World,
    ) -> Result<(), NodeRunError> {
        // Get the pipeline
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline.0) else {
            return Ok(());
        };

        // Post process write
        let post_process = view_target.post_process_write();

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("show prepass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Viewport
        if let Some(viewport) = camera.viewport.as_ref() {
            render_pass.set_camera_viewport(viewport);
        }

        // Draw
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group.0, &[uniform_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Component)]
struct CachedShowPrepassPipeline(CachedRenderPipelineId);

fn prepare_pipelines(
    mut commands: Commands,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ShowPrepassPipeline>>,
    pipeline: Res<ShowPrepassPipeline>,
    views: Query<(Entity, &ExtractedView, &ShowPrepass, Option<&Msaa>)>,
) {
    for (view_entity, view, show_prepass, msaa) in &views {
        let key = ShowPrepassPipelineKey {
            show_prepass: *show_prepass,
            hdr: view.hdr,
            multisampled: msaa.is_some_and(|msaa| msaa.samples() > 1),
        };
        let pipeline = pipelines.specialize(&pipeline_cache, &pipeline, key);

        commands
            .entity(view_entity)
            .insert(CachedShowPrepassPipeline(pipeline));
    }
}

#[derive(Component)]
struct ShowPrepassBindGroup(BindGroup);

fn prepare_bind_groups(
    mut commands: Commands,
    views: Query<(
        Entity,
        &ShowPrepass,
        Option<&ViewPrepassTextures>,
        Option<&Msaa>,
    )>,
    uniforms: Res<ComponentUniforms<ShowPrepassUniform>>,
    render_device: ResMut<RenderDevice>,
    pipeline: Res<ShowPrepassPipeline>,
) {
    for (view_entity, show_prepass, view_prepass_textures, msaa) in views {
        let key = ShowPrepassPipelineLayoutKey {
            show_prepass: *show_prepass,
            multisampled: msaa.is_some_and(|msaa| msaa.samples() > 1),
        };
        let layout = pipeline.layouts.get(&key).unwrap();

        let Some(uniform) = uniforms.uniforms().binding() else {
            continue;
        };
        let uniform_entry = BindGroupEntry {
            binding: 0,
            resource: uniform,
        };

        let resource = match show_prepass {
            ShowPrepass::Depth => view_prepass_textures
                .and_then(|t| t.depth_view())
                .map(|v| v.into_binding()),
            ShowPrepass::Normals => view_prepass_textures
                .and_then(|t| t.normal_view())
                .map(|v| v.into_binding()),
            ShowPrepass::MotionVectors => view_prepass_textures
                .and_then(|t| t.motion_vectors_view())
                .map(|v| v.into_binding()),
        };
        let Some(resource) = resource else {
            warn_once!("Requested prepass texture for {show_prepass:?} is missing; skipping.");

            commands
                .entity(view_entity)
                .remove::<ShowPrepassBindGroup>();
            continue;
        };
        let texture_entry = BindGroupEntry {
            binding: 1,
            resource,
        };

        let bind_group = render_device.create_bind_group(
            "show_prepass_bind_group",
            layout,
            &[uniform_entry, texture_entry],
        );

        commands
            .entity(view_entity)
            .insert(ShowPrepassBindGroup(bind_group));
    }
}
