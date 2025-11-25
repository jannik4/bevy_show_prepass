use bevy::{
    core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass, NormalPrepass},
    prelude::*,
};
use bevy_show_prepass::*;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Window {
                title: "Bevy Show Prepass".to_string(),
                fit_canvas_to_parent: true,
                ..default()
            }
            .into(),
            ..default()
        }),
        // Add the ShowPrepassPlugin
        ShowPrepassPlugin,
    ));

    // Setup and Update
    app.add_systems(Startup, setup);
    app.add_systems(Update, (move_cube, choose_show_prepass_mode));

    // Run the app
    app.run()
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera setup
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 6.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Add all prepasses to the camera
        DepthPrepass,
        NormalPrepass,
        MotionVectorPrepass,
        // Optionally scale the depth visualization, e.g. depth = depth^0.75
        ShowPrepassDepthPower(0.75),
    ));

    // Scene setup
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5))),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_xyz(2.0, 0.5, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(-2.0, 0.5, 0.0),
        Cube,
    ));
    commands.spawn((
        SpotLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // UI
    commands.spawn((
        Text(
            "(1): Show view\n\
             (2): Show depth prepass\n\
             (3): Show normal prepass\n\
             (4): Show motion vector prepass"
                .to_string(),
        ),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));
}

// Move the cube over time
fn move_cube(mut cube: Single<&mut Transform, With<Cube>>, time: Res<Time>) {
    cube.translation.z = time.elapsed_secs().sin() * 2.0;
}

// Choose which prepass to show with number keys 1-4
fn choose_show_prepass_mode(
    mut commands: Commands,
    camera: Single<Entity, With<Camera3d>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        commands.entity(*camera).remove::<ShowPrepass>();
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        commands.entity(*camera).insert(ShowPrepass::Depth);
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        commands.entity(*camera).insert(ShowPrepass::Normals);
    } else if keyboard.just_pressed(KeyCode::Digit4) {
        commands.entity(*camera).insert(ShowPrepass::MotionVectors);
    }
}
