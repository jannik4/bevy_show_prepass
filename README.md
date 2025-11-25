# bevy_show_prepass

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Build Status](https://github.com/jannik4/bevy_show_prepass/workflows/CI/badge.svg)](https://github.com/jannik4/bevy_show_prepass/actions)
[![crates.io](https://img.shields.io/crates/v/bevy_show_prepass.svg)](https://crates.io/crates/bevy_show_prepass)
[![docs.rs](https://img.shields.io/badge/docs-latest-blue.svg)](https://docs.rs/bevy_show_prepass)

A Bevy plugin to visualize depth, normal and motion vector prepasses.

## Usage

For a complete example, see the [simple example](https://github.com/jannik4/bevy_show_prepass/blob/main/examples/simple.rs).

```rust,ignore
// Add the plugin
app.add_plugins(ShowPrepassPlugin);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Add the desired prepasses to the camera (DepthPrepass, NormalPrepass, MotionVectorPrepass)
        DepthPrepass,
        // Show the desired prepass (ShowPrepass::Depth, ShowPrepass::Normals, ShowPrepass::MotionVector)
        ShowPrepass::Depth,
        // Optionally scale the depth visualization, e.g. depth = depth^0.75
        ShowPrepassDepthPower(0.75),
    ));
}
```

## Development

Run example with shader hot reloading:

```sh
cargo run --example simple --features bevy/embedded_watcher
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE-2.0](LICENSE-Apache-2.0) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
