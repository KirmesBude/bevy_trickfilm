//! This module handles loading an AnimationClipSet2D and AnimationClip2D from a manifest file.
//! Look at the manifest type declarations and the examples on how to write such a manidest file.
//!

use std::ops::Range;

use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::Handle,
    utils::{thiserror, BoxedFuture, HashMap},
};
use serde::Deserialize;
use thiserror::Error;

use super::{AnimationClip2D, AnimationClip2DError, Trickfilm};

/// Loader for spritesheet animation manifest files written in ron. Loads an SpriteSheetAnimationSet asset.
#[derive(Default)]
pub struct Animation2DLoader;

/// Possible errors that can be produced by [`Animation2DLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Animation2DLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not open file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    /// An [`AnimationClip2DError`]
    #[error("AnimationClip2D has internal erro: {0}")]
    AnimationClip2DError(#[from] AnimationClip2DError),
}

/// Declaration of the deserialized variant for the animation frame indices.
#[derive(Debug, Deserialize)]
pub enum AnimationClip2DKeyframesManifest {
    /// You can specify the index of each frame seperately.
    KeyframeVec(Vec<usize>),
    /// Use this, if the animation frames of an animation have continuous indices.
    KeyframeRange(Range<usize>),
}

impl From<AnimationClip2DKeyframesManifest> for Vec<usize> {
    fn from(manifest: AnimationClip2DKeyframesManifest) -> Self {
        match manifest {
            AnimationClip2DKeyframesManifest::KeyframeVec(vec) => vec,
            AnimationClip2DKeyframesManifest::KeyframeRange(range) => range.collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AnimationClip2DManifest {
    name: String,
    keyframes: AnimationClip2DKeyframesManifest,
    #[serde(default)]
    keyframe_timestamps: Option<Vec<f32>>,
    duration: f32,
}

/// File extension for spritesheet animation manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["trickfilm"];

impl AssetLoader for Animation2DLoader {
    type Asset = Trickfilm;
    type Settings = ();
    type Error = Animation2DLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let animation_clip_manifest_set =
                ron::de::from_bytes::<Vec<AnimationClip2DManifest>>(&bytes)?;

            let animations: Result<HashMap<String, Handle<AnimationClip2D>>, AnimationClip2DError> =
                animation_clip_manifest_set
                    .into_iter()
                    .map(|animation_clip_manifest| {
                        let name = animation_clip_manifest.name;
                        let duration = animation_clip_manifest.duration;
                        let keyframes: Vec<usize> = animation_clip_manifest.keyframes.into();
                        let keyframe_timestamps =
                            animation_clip_manifest.keyframe_timestamps.unwrap_or(
                                (0..keyframes.len())
                                    .map(|i| {
                                        let i = i as f32 / keyframes.len() as f32;
                                        i * duration
                                    })
                                    .collect(),
                            );

                        let animation_clip =
                            AnimationClip2D::new(keyframe_timestamps, keyframes, duration)?;
                        Ok((
                            name.clone(),
                            load_context.add_labeled_asset(name, animation_clip),
                        ))
                    })
                    .collect();

            let trickfilm = Trickfilm {
                animations: animations?,
            };
            Ok(trickfilm)
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}
