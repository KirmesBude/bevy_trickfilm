//! This crate allows you to directly load a TextureAtlas from a manifest file.
//!
//! `bevy_titan` introduces a [`SpriteSheetManifest`](crate::SpriteSheetManifest) and the corresponding [`SpriteSheetLoader`](crate::SpriteSheetLoader).
//! Assets with the 'titan' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`TextureAtlas`](::bevy::sprite::TextureAtlas) [`Handle`](::bevy::asset::Handle).
//!
//! ### `spritesheet.titan`
//! ```rust,ignore
//! SpriteSheetManifest ( /* The explicit type name can be omitted */
//!     path: String, /* path to spritesheet image asset */
//!     tile_size: (
//!         w: f32,
//!         h: f32,
//!     ),
//!     columns: usize,
//!     rows: usize,
//!    // These can be optionally defined
//!    /*
//!    padding: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    offset: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    */
//! )
//! ```
//!
//! ```edition2021
//! # use bevy_titan::SpriteSheetLoaderPlugin;
//! # use bevy::prelude::*;
//! #
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(SpriteSheetLoaderPlugin)
//!         .add_system(load_spritesheet)
//!         .run();
//! }
//!
//! fn load_spritesheet(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let texture_atlas_handle = asset_server.load("spritesheet.titan");
//!     commands.spawn(Camera2dBundle::default());
//!     commands.spawn(
//!         SpriteSheetBundle {
//!              texture_atlas: texture_atlas_handle,
//!              transform: Transform::from_scale(Vec3::splat(6.0)),
//!              ..default()
//!         }
//!     );
//! }
//!
//! ```


#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use animation::{animation_update_internal, animate_sprite};
use asset_loader::{SpriteSheetAnimationLoader, SpriteSheetAnimationSet};
use bevy::prelude::{AddAsset, App, Plugin, IntoSystemDescriptor};

pub mod asset_loader;
pub mod animation;

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct SpriteSheetAnimationLoaderPlugin;

impl Plugin for SpriteSheetAnimationLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_asset::<SpriteSheetAnimationSet>()
        .init_asset_loader::<SpriteSheetAnimationLoader>()
        .add_system(animation_update_internal)
        .add_system(animate_sprite.after(animation_update_internal));
    }
}

pub mod prelude {
    pub use crate::asset_loader::{SpriteSheetAnimationSet, SpriteSheetAnimation};
    pub use crate::animation::SpriteSheetAnimationPlayer;
    pub use crate::SpriteSheetAnimationLoaderPlugin;
}
