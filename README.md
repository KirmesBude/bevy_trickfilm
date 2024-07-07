# bevy_trickfilm

[![crates.io](https://img.shields.io/crates/v/bevy_trickfilm)](https://crates.io/crates/bevy_trickfilm)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![docs.rs](https://docs.rs/bevy_trickfilm/badge.svg)](https://docs.rs/bevy_trickfilm)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)

| bevy | bevy_trickfilm |
|------|----------------|
| main | main           |
| 0.14 | 0.7.0          |
| 0.13 | 0.6.0          |
| 0.12 | 0.4.0, 0.5.0   |
| 0.11 | 0.3.0          |
| 0.10 | 0.2.0          |
| 0.9  | 0.1.0          |

## What is bevy_trickfilm?

Simple plugin to load spritesheet animations from manifest files written in ron. The animations are not directly tied to a certain sprite sheet.
You can combine this with plugins that add the ability to load a texture atlas from a manifest file. For example: [bevy_titan](https://github.com/KirmesBude/bevy_titan) or [bevy_heterogeneous_texture_atlas_loader](https://github.com/ickshonpe/bevy_heterogeneous_texture_atlas_loader).

## Quickstart


```toml, ignore
# In your Cargo.toml
bevy_trickfilm = "0.7"
```

### animation_clip.trickfilm
```rust, ignore
//! A basic example of a trickfilm file.
{
    "idle": (
        keyframes: KeyframesRange((start: 0, end: 4)),
        duration: 1.0,
    ),
    "run": (
        keyframes: KeyframesRange((start: 4, end: 10)),
        duration: 0.6,
    ),
    "jump": (
        keyframes: KeyframesVec([10,11,12]),
	    keyframe_timestamps: Some([0.0. 1.0, 3.0]),
        duration: 0.4,
    ),
}
```

### main.rs
```rust, ignore
//! A basic example of how to load an AnimationClip2D asset from a trickfilm file
//! and play the animation clip.
use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Animation2DPlugin)
        .add_systems(Startup, (setup, load_texture_atlas).chain())
        .add_systems(Update, play_animation_once_loaded)
        .run();
}

fn setup() {
    /* Setup camera and other stuff */
}

fn load_texture_atlas(mut commands: Commands) {
    let texture_handle = /* Create your TextureAtlas and retrieve a handle to it */;
    let layout_handle = /* Create your TextureAtlas and retrieve a handle to it */;

    commands
        .spawn(SpriteBundle {
            texture: texture_handle,
            ..Default::default()
        })
        .insert(TextureAtlas {
            layout: layout_handle,
            ..Default::default()
        })
        .insert(AnimationPlayer2D::default());
}

// Once the scene is loaded, start the animation
fn play_animation_once_loaded(
    asset_server: Res<AssetServer>
    mut players: Query<&mut AnimationPlayer2D, Added<AnimationPlayer2D>>,
) {
    for mut player in &mut players {
        player.start(asset_server.load("animation_clip.trickfilm#idle")).repeat();
    }
}
```

## Documentation

[Full API Documentation](https://docs.rs/bevy_trickfilm)

[File format specifiction](https://github.com/KirmesBude/bevy_trickfilm/blob/main/docs/FileFormatSpecification.md)

[Examples](https://github.com/KirmesBude/bevy_trickfilm/tree/main/examples)

## Future Work

* Not sure

## License

bevy_trickfilm is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](https://github.com/KirmesBude/bevy_trickfilm/blob/main/LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/KirmesBude/bevy_trickfilm/blob/main/LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

Some of the code was adapted from other sources.
The [assets](https://github.com/KirmesBude/bevy_trickfilm/tree/main/assets) included in this repository fall under different open licenses.
See [CREDITS.md](https://github.com/KirmesBude/bevy_trickfilm/blob/main/CREDITS.md) for the details of the origin of the adapted code and licenses of those files.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
