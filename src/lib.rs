//! This crate allows you to directly load an [`AnimationClip2DSet`](crate::asset::AnimationClip2DSet) and/or [`AnimationClip2D`](crate::asset::AnimationClip2D) from a manifest file and play animations.
//!
//! `bevy_trickfilm` introduces an ron file with trickfilm extension (contains vector of [`TrickfilmEntry`](crate::asset::asset_loader::TrickfilmEntry)) and the corresponding [`Animation2DLoader`](crate::asset::asset_loader::Animation2DLoader).
//! Assets with the 'trickfilm' extension can be loaded just like any other asset via the [`AssetServer`](bevy::asset::AssetServer)
//! and will yield an [`AnimationClip2DSet`](crate::asset::AnimationClip2DSet) [`Handle`](bevy::asset::Handle) (or an [`AnimationClip2D`](crate::asset::AnimationClip2D) [`Handle`](bevy::asset::Handle) directly via labeled assets).
//! Additionally it provides built-in support for animation playing with the [`AnimationPlayer2D`](crate::animation::AnimationPlayer2D) component.
//!
//! ### `gabe-idle-run.trickfilm`
//! ```rust,ignore
//! [
//!     (
//!         name: "idle",                                           /* Name for this animation clip. */
//!         keyframes: KeyframesVec([0]),                           /* Keyframes of this animation. Indices inside of the corresponding TextureAtlas that represent the individual keyframes. Must be the same length as keyframe_timestamps. */
//!         keyframe_timestamps: Some([0.0]),                       /* Keyframe timestamps of this animation. You can provide None; see below. Must be the same length as keyframes.*/
//!         duration: 0.1,                                          /* Complete duration of the animation. Must be greater than the maximum value of keyframe_timestamps. */
//!     ),
//!     (
//!         name: "run",
//!         keyframes: KeyframesRange((start: 1, end: 7)),          /* If the indices of your keyframes are in order, you can simply provide a range instead. */
//!         keyframe_timestamps: None,                              /* Provide None to automatically calulcate the timestamps based on the amount of keyframes and the duration. */
//!         duration: 0.6,
//!     ),
//! ]
//! ```
//!
//! ```edition2021
//! # use bevy_trickfilm::prelude::*;
//! # use bevy::prelude::*;
//! #
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         /* Add some plugin to load spritesheet manifest files */
//!         .add_plugins(Animation2DPlugin)
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, setup_scene_once_loaded)
//!         .run();
//! }
//!
//! #[derive(Resource)]
//! struct Animations(Vec<Handle<AnimationClip2D>>);
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     // Insert a resource with the current animation information
//!     commands.insert_resource(Animations(vec![
//!         asset_server.load("sprite_animation/gabe-idle-run.trickfilm#run"),
//!     ]));
//!
//!     // Camera
//!     commands.spawn(Camera2dBundle::default());
//!
//!     // SpriteSheet entity
//!     commands
//!         .spawn(SpriteBundle {
//!             transform: Transform::from_scale(Vec3::splat(6.0)),
//!             ..default()
//!         })
//!         .insert(AnimationPlayer2D::default());
//! }
//!
//! fn setup_scene_once_loaded(
//!     animations: Res<Animations>,
//!     mut player: Query<&mut AnimationPlayer2D>,
//!     mut done: Local<bool>,
//! ) {
//!     if !*done {
//!         if let Ok(mut player) = player.get_single_mut() {
//!             player.play(animations.0[0].clone_weak()).repeat();
//!             *done = true;
//!         }
//!     }
//! }
//!
//! ```
//!

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::prelude::{App, Plugin};

pub mod animation;
pub mod asset;

/// Adds support for 2d animation loading and playing.
pub struct Animation2DPlugin;

impl Plugin for Animation2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            asset::Animation2DLoaderPlugin,
            animation::AnimationPlayer2DPlugin,
        ));
    }
}

/// `use bevy_trickfilm::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::animation::{AnimationPlayer2D, AnimationPlayer2DPlugin};
    pub use crate::asset::{Animation2DLoaderPlugin, AnimationClip2D, AnimationClip2DSet};
    pub use crate::Animation2DPlugin;
}
