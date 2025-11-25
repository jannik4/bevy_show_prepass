#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var<uniform> show_prepass: ShowPrepassUniform;

#ifdef SHOW_DEPTH
#ifdef MULTISAMPLED
@group(0) @binding(1) var depth_prepass_texture: texture_depth_multisampled_2d;
#else
@group(0) @binding(1) var depth_prepass_texture: texture_depth_2d;
#endif
#endif

#ifdef SHOW_NORMALS
#ifdef MULTISAMPLED
@group(0) @binding(1) var normal_prepass_texture: texture_multisampled_2d<f32>;
#else
@group(0) @binding(1) var normal_prepass_texture: texture_2d<f32>;
#endif
#endif

#ifdef SHOW_MOTION_VECTORS
#ifdef MULTISAMPLED
@group(0) @binding(1) var motion_vector_prepass_texture: texture_multisampled_2d<f32>;
#else
@group(0) @binding(1) var motion_vector_prepass_texture: texture_2d<f32>;
#endif
#endif

struct ShowPrepassUniform {
    depth_power: f32,
    delta_time: f32,
}

@fragment
fn fragment(
    #ifdef MULTISAMPLED
    @builtin(sample_index) sample_index: u32,
    #endif
    in: FullscreenVertexOutput
) -> @location(0) vec4<f32> {
    #ifdef SHOW_DEPTH
    #ifdef MULTISAMPLED
    var depth = textureLoad(depth_prepass_texture, vec2<i32>(in.position.xy), i32(sample_index));
    #else
    var depth = textureLoad(depth_prepass_texture, vec2<i32>(in.position.xy), 0);
    #endif

    depth = pow(depth, show_prepass.depth_power);
    return vec4(depth, depth, depth, 1.0);
    #endif
    
    #ifdef SHOW_NORMALS
    #ifdef MULTISAMPLED
    let normal = textureLoad(normal_prepass_texture, vec2<i32>(in.position.xy), i32(sample_index));
    #else
    let normal = textureLoad(normal_prepass_texture, vec2<i32>(in.position.xy), 0);
    #endif

    return normal;
    #endif
    
    #ifdef SHOW_MOTION_VECTORS
    #ifdef MULTISAMPLED
    let motion_vector = textureLoad(motion_vector_prepass_texture, vec2<i32>(in.position.xy), i32(sample_index)).rg;
    #else
    let motion_vector = textureLoad(motion_vector_prepass_texture, vec2<i32>(in.position.xy), 0).rg;
    #endif

    return vec4(abs(motion_vector) / show_prepass.delta_time, 0.0, 1.0);
    #endif
}
