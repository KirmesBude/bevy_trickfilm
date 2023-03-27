//! This crate allows you to directly load an [`AnimationClipSet2D`](crate::asset::AnimationClipSet2D) and/or [`AnimationClip2D`](crate::asset::AnimationClip2D) from a manifest file and play animations.
//!
//! `bevy_trickfilm` introduces an [`AnimationClipSet2DManifest`](crate::asset::asset_loader::AnimationClipSet2DManifest) (contains [`AnimationClip2DManifest`](crate::asset::asset_loader::AnimationClip2DManifest)) and the corresponding [`Animation2DLoader`](crate::asset::asset_loader::Animation2DLoader).
//! Assets with the 'trickfilm' extension can be loaded just like any other asset via the [`AssetServer`](bevy::asset::AssetServer)
//! and will yield an [`AnimationClipSet2D`](crate::asset::AnimationClipSet2D) [`Handle`](bevy::asset::Handle) (or an [`AnimationClip2D`](crate::asset::AnimationClip2D) [`Handle`](bevy::asset::Handle) directly via labeled assets).
//! Additionally it provides built-in support for animation playing with the [`AnimationPlayer2D`](crate::animation::AnimationPlayer2D) component.
//!
//! ### `gabe-idle-run.trickfilm`
//! ```rust,ignore
//! AnimationClipSet2DManifest (                                    /* The explicit type name can be omitted. */
//!    name: String,                                                /* Optional name for this animation set. */
//!    animations: {
//!         "idle": (                                               /* Name of the animation. */
//!             keyframe_timestamps: Some([0]),                     /* Keyframe timestamps of this animation. You can provide None, to automatically calulcate the timestamps based on the amount of keyframes and the duration. */
//!             keyframes: SpriteSheet(                             /* Keyframes of this animation. For the SpriteSheet variant you need to provide: */
//!                 "spritesheet_animation/gabe-idle-run.titan",    /* A path to the manifest file that will load to a TextureAtlas asset. */
//!                 IndexVec(                                       /* The indices inside of that TextureAtlas that represent the individual keyframes. */
//!                     [0]
//!                 ),
//!             ),
//!             duration: 0.1,                                      /* Complete duration of the animation. */
//!         ),
//!         "run": (
//!             keyframe_timestamps: None,                          /* Will automatically calculate the timestamps. */
//!             keyframes: SpriteSheet(     
//!                 "spritesheet_animation/gabe-idle-run.titan",
//!                 IndexRange(                                     /* If the indices of your keyframes are in order, you can simply provide a range instead. */
//!                     (start: 1, end: 7)
//!                 ),
//!             ),
//!             duration: 0.6,
//!         ),
//!    },
//! )
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
//!         .add_plugin(Animation2DPlugin)
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let animation_clip_handle = asset_server.load("spritesheet_animation.trickfilm#run");
//!     let animation_player = AnimationPlayer2D::default().play(animation_clip_handle).repeat();
//!     commands.spawn(Camera2dBundle::default());
//!     commands.spawn((
//!         SpriteSheetBundle {
//!             transform: Transform::from_scale(Vec3::splat(6.0)),
//!             ..default()
//!         },
//!         animation_player,
//!     ));
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
        app.add_plugin(asset::Animation2DLoaderPlugin)
            .add_plugin(animation::AnimationPlayer2DPlugin);
    }
}

/// `use bevy_trickfilm::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::animation::{AnimationPlayer2D, AnimationPlayer2DPlugin};
    pub use crate::asset::{Animation2DLoaderPlugin, AnimationClip2D, AnimationClipSet2D};
    pub use crate::Animation2DPlugin;
}
