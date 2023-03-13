//! This module handles loading a SpriteSheetAnimationSet from a manifest file.
//!
//! `bevy_trickfilm::asset_loader` introduces a [`SpriteSheetAnimationSetManifest`](crate::asset_loader::SpriteSheetAnimationSetManifest) (contains [`SpriteSheetAnimationManifest`](crate::asset_loader::SpriteSheetAnimationManifest)) and the corresponding [`SpriteSheetAnimationLoader`](crate::asset_loader::SpriteSheetAnimationLoader).
//! Assets with the 'trickfilm' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`SpriteSheetAnimationSet`](crate::asset_loader::SpriteSheetAnimationSet) [`Handle`](::bevy::asset::Handle).
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
//!         .run();
//! }
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
//!     ));
//! }
//!
//! ```

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::{AddAsset, App, Handle, Plugin},
    reflect::TypeUuid,
    sprite::TextureAtlas,
    utils::{BoxedFuture, HashMap, HashSet},
};
use serde::Deserialize;
use std::{ops::Range, path::PathBuf};

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct SpriteSheetAnimationLoaderPlugin;

impl Plugin for SpriteSheetAnimationLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<SpriteSheetAnimationSet>()
            .init_asset_loader::<SpriteSheetAnimationLoader>();
    }
}

/// Loader for spritesheet animation manifest files written in ron. Loads an SpriteSheetAnimationSet asset.
#[derive(Default)]
pub struct SpriteSheetAnimationLoader;

/// File extension for spritesheet animation manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["trickfilm"];

impl AssetLoader for SpriteSheetAnimationLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let spritesheet_animationset_manifest =
                ron::de::from_bytes::<SpriteSheetAnimationSetManifest>(bytes)?;

            let mut spritesheet_animationset = SpriteSheetAnimationSet {
                name: spritesheet_animationset_manifest.name,
                ..Default::default()
            };
            let mut dependencies = HashSet::new();
            for (animation_name, spritesheet_animation_manifest) in
                spritesheet_animationset_manifest.animations
            {
                let spritesheet_animation_asset_path =
                    AssetPath::new(PathBuf::from(&spritesheet_animation_manifest.path), None);
                dependencies.insert(spritesheet_animation_asset_path.clone());

                let texture_atlas_handle: Handle<TextureAtlas> =
                    load_context.get_handle(spritesheet_animation_asset_path.clone());
                let spritesheet_animation = SpriteSheetAnimation {
                    texture_atlas_handle,
                    repeating: spritesheet_animation_manifest.repeating,
                    fps: spritesheet_animation_manifest.fps,
                    indices: match spritesheet_animation_manifest.indices {
                        AnimationFrameIndices::IndexVec(vec) => vec,
                        AnimationFrameIndices::IndexRange(range) => range.collect(),
                    },
                };
                spritesheet_animationset
                    .animations
                    .insert(animation_name, spritesheet_animation);
            }

            let mut spritesheet_animation_asset = LoadedAsset::new(spritesheet_animationset);
            for dependency in dependencies {
                spritesheet_animation_asset.add_dependency(dependency);
            }

            load_context.set_default_asset(spritesheet_animation_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

/// Declaration of the deserialized variant for the animation frame indices.
#[derive(Debug, Deserialize)]
pub enum AnimationFrameIndices {
    /// You can specify the index of each frame seperately.
    IndexVec(Vec<usize>),
    /// Use this, if the animation frames of an animation have continuous indices.
    IndexRange(Range<usize>),
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Deserialize)]
pub struct SpriteSheetAnimationSetManifest {
    /// Optional name of this animation set.
    #[serde(default)]
    pub name: Option<String>,
    /// A map of all animations in this set, identified by their names.
    pub animations: HashMap<String, SpriteSheetAnimationManifest>,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Deserialize)]
pub struct SpriteSheetAnimationManifest {
    /// Path to a manifest files that loads a TextureAtlas that houses all frames of this animation.
    pub path: String,
    /// If set, the animation will loop.
    pub repeating: bool,
    /// Animation speed in frames per second.
    pub fps: usize,
    /// An ordered list of incides of the TextureAtlas that represent the frames of this animation.
    pub indices: AnimationFrameIndices,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Default, TypeUuid)]
#[uuid = "ec942212-87dc-4ee4-8300-1e160a389c37"]
pub struct SpriteSheetAnimationSet {
    /// Optional name of this animation set.
    pub name: Option<String>,
    /// A map of all animations in this set, identified by their names.
    pub animations: HashMap<String, SpriteSheetAnimation>,
}

/* TODO: Extend repeating to some kind of Mode, that supports Once, Repeating and PingPong */
/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Default)]
pub struct SpriteSheetAnimation {
    /// The texture atlas that houses all frames of this animation.
    pub texture_atlas_handle: Handle<TextureAtlas>,
    /// If set, the animation will loop.
    pub repeating: bool,
    /// Animation speed in frames per second.
    pub fps: usize,
    /// An ordered list of incides of the TextureAtlas that represent the frames of this animation.
    pub indices: Vec<usize>,
}
