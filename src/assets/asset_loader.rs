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
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::{BoxedFuture, HashMap},
};
use serde::Deserialize;
use std::ops::Range;

use super::{AnimationClip2D, AnimationClipSet2D};

/// Loader for spritesheet animation manifest files written in ron. Loads an SpriteSheetAnimationSet asset.
#[derive(Default)]
pub struct Animation2DLoader;

/// File extension for spritesheet animation manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["trickfilm"];

impl AssetLoader for Animation2DLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let animation_clip_set_manifest =
                ron::de::from_bytes::<AnimationClipSet2DManifest>(bytes)?;

            // Convert AnimationClipSet2DManifest to AnimationClipSet2D
            let animation_clip_set = AnimationClipSet2D {
                name: animation_clip_set_manifest.name,
                animations: animation_clip_set_manifest
                    .animations
                    .into_iter()
                    .map(|(name, animation_clip_manifest)| {
                        // Convert AnimationClip2DManifest to AnimationClip2D
                        let animation_clip = AnimationClip2D {
                            // If keyframe timestamps are not provided we calculate equal distance points across the duration for each keyframe
                            keyframe_timestamps: match animation_clip_manifest.keyframe_timestamps {
                                Some(vec) => vec,
                                None => (0..animation_clip_manifest.keyframes.len())
                                    .map(|i| {
                                        (i as f32) * animation_clip_manifest.duration
                                            / (animation_clip_manifest.keyframes.len() as f32)
                                    })
                                    .collect(),
                            },
                            keyframes: match &animation_clip_manifest.keyframes {
                                Keyframes2DManifest::SpriteSheet(path, indices) => {
                                    super::Keyframes2D::SpriteSheet(
                                        load_context.get_handle(path),
                                        indices.clone().into(),
                                    )
                                }
                                Keyframes2DManifest::Sprite(paths) => super::Keyframes2D::Sprite(
                                    paths
                                        .iter()
                                        .map(|path| load_context.get_handle(path))
                                        .collect(),
                                ),
                            },
                            duration: animation_clip_manifest.duration,
                        };

                        let mut animation_clip_asset = LoadedAsset::new(animation_clip);
                        match animation_clip_manifest.keyframes {
                            Keyframes2DManifest::SpriteSheet(path, _) => {
                                animation_clip_asset.add_dependency(path.into())
                            }
                            Keyframes2DManifest::Sprite(paths) => paths
                                .iter()
                                .for_each(|path| animation_clip_asset.add_dependency(path.into())),
                        }
                        (
                            name.clone(),
                            load_context.set_labeled_asset(&name, animation_clip_asset),
                        )
                    })
                    .collect(),
            };

            // TODO: Does this need all dependencies as well?
            let animation_clip_set_asset = LoadedAsset::new(animation_clip_set);
            load_context.set_default_asset(animation_clip_set_asset);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

/// Declaration of the deserialized variant for the animation frame indices.
#[derive(Debug, Clone, Deserialize)]
pub enum SpriteSheetIndices {
    /// You can specify the index of each frame seperately.
    IndexVec(Vec<usize>),
    /// Use this, if the animation frames of an animation have continuous indices.
    IndexRange(Range<usize>),
}

impl From<SpriteSheetIndices> for Vec<usize> {
    fn from(indices: SpriteSheetIndices) -> Self {
        match indices {
            SpriteSheetIndices::IndexVec(vec) => vec,
            SpriteSheetIndices::IndexRange(range) => range.collect(),
        }
    }
}

impl SpriteSheetIndices {
    /// Returns the number of elements in the indices definition, also referred to as its 'length'.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            SpriteSheetIndices::IndexVec(vec) => vec.len(),
            SpriteSheetIndices::IndexRange(range) => range.len(),
        }
    }
}

/// Declaration of the deserialized variant for the animation keyframes.
#[derive(Debug, Deserialize)]
pub enum Keyframes2DManifest {
    /// For Spritesheet animations
    SpriteSheet(String, SpriteSheetIndices),
    /// For Sprite animations
    Sprite(Vec<String>),
}

impl Keyframes2DManifest {
    /// Returns the number of elements in the keyframe manifest, also referred to as its 'length'.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Keyframes2DManifest::SpriteSheet(_, indices) => indices.len(),
            Keyframes2DManifest::Sprite(vec) => vec.len(),
        }
    }
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Deserialize)]
pub struct AnimationClip2DManifest {
    /// Timestamp for each keyframe. If set to None, timestamps will be generated from number of keyframes and duration.
    pub keyframe_timestamps: Option<Vec<f32>>,
    /// An ordered list of incides of the TextureAtlas or Images that represent the frames of this animation.
    pub keyframes: Keyframes2DManifest,
    /// Total duration of this animation clip.
    pub duration: f32,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Deserialize)]
pub struct AnimationClipSet2DManifest {
    /// Optional name of this animation set.
    #[serde(default)]
    pub name: Option<String>,
    /// A map of all animations in this set, identified by their names.
    pub animations: HashMap<String, AnimationClip2DManifest>,
}
