//! This crate allows you to directly load a SpriteSheetAnimationSet from a manifest file and play animations.
//!
//! `bevy_trickfilm` introduces a [`SpriteSheetAnimationSetManifest`](crate::asset_loader::SpriteSheetAnimationSetManifest) (contains [`SpriteSheetAnimationManifest`](crate::asset_loader::SpriteSheetAnimationManifest)) and the corresponding [`SpriteSheetAnimationLoader`](crate::asset_loader::SpriteSheetAnimationLoader).
//! Assets with the 'trickfilm' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`SpriteSheetAnimationSet`](crate::asset_loader::SpriteSheetAnimationSet) [`Handle`](::bevy::asset::Handle).
//! Additionally it provides built-in support for animation playing with the [`SpriteSheetAnimationPlayer`](crate::animation::SpriteSheetAnimationPlayer) component.
//!
//! ### `spritesheet_animation.trickfilm`
//! ```rust,ignore
//! SpriteSheetAnimationSetManifest (       /* The explicit type name can be omitted */
//!    name: String,                        /* Optional name for this animation set */
//!    animations: {
//!        "idle": (                        /* Name of the animation */
//!            path: String,                /* Path to some file that support loading to TextureAtlas (such as manifest files for bevy_titan or bevy_heterogeneous_texture_atlas_loader) */
//!            repeating: boolean,          /* Whether this animation shall automatically repeat from the start */
//!            fps: usize,                  /* Animation speed in frames per second */
//!            indices: [0,1,2,3],          /* Indices into the TextureAtlas that represent the ordered list of frames of this animation */
//!        ),
//!        "kick": (                         
//!            path: String,                /* Animation of the same AnimationSet can reference the same or a different underlying spritesheet */
//!            repeating: boolean,          
//!            fps: usize,                  
//!            indices: [4,5,6,7,8,9,10],    
//!        ),
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
//!         .add_plugin(SpriteSheetAnimationPlugin)
//!         .add_startup_system(setup)
//!         .add_system(kick)
//!         .run();
//! }
//!
//! #[derive(Component)]
//! struct Controlled;
//!
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let spritesheet_animationset_handle = asset_server.load("spritesheet_animation.trickfilm");
//!     commands.spawn(Camera2dBundle::default());
//!     commands.spawn((
//!         SpriteSheetBundle {
//!             transform: Transform::from_scale(Vec3::splat(6.0)),
//!             ..default()
//!         },
//!         SpriteSheetAnimationPlayer::new(spritesheet_animationset_handle)
//!             .with_animation(String::from("idle")),
//!         Controlled,
//!     ));
//! }
//!
//! fn kick(mut animation_players: Query<&mut SpriteSheetAnimationPlayer, With<Controlled>>, keys: Res<Input<KeyCode>>) {
//!     if keys.just_pressed(KeyCode::Space) {
//!         for mut animation_player in &mut animation_players {
//!             animation_player.play(String::from("kick"));
//!         }
//!     }
//! }
//!
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::prelude::{App, Plugin};

pub mod animation;
pub mod assets;

/// Adds support for spritesheet animation loading and playing.
pub struct SpriteSheetAnimationPlugin;

impl Plugin for SpriteSheetAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(assets::Animation2DLoaderPlugin)
            .add_plugin(animation::AnimationPlayer2DPlugin);
    }
}

/// `use bevy_trickfilm::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::animation::{AnimationPlayer2D, AnimationPlayer2DPlugin};
    pub use crate::assets::{Animation2DLoaderPlugin, AnimationClip2D, AnimationClipSet2D};
    pub use crate::SpriteSheetAnimationPlugin;
}
