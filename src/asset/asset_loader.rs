//! This module contains the internals of the Animation2DLoader.
//!

use std::ops::Range;

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::{AppFunctionRegistry, FromWorld, Handle, World},
    reflect::Reflect,
    utils::HashMap,
};
use serde::Deserialize;
use thiserror::Error;

use super::{AnimationClip2D, AnimationClip2DError, AnimationClip2DSet};

pub(crate) struct Animation2DLoader {
    function_registry: AppFunctionRegistry,
}

impl FromWorld for Animation2DLoader {
    fn from_world(world: &mut World) -> Self {
        Self {
            function_registry: world.resource::<AppFunctionRegistry>().clone(),
        }
    }
}

/// Possible errors that can be produced by Animation2DLoader.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Animation2DLoaderError {
    /// An [IOError](std::io::Error).
    #[error("Could not open file: {0}")]
    Io(#[from] std::io::Error),
    /// A [SpannedError](ron::error::SpannedError).
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    /// An [`AnimationClip2DError`].
    #[error("AnimationClip2D has internal error: {0}")]
    AnimationClip2DError(#[from] AnimationClip2DError),
}

/// Declaration of the deserialized variant for the animation frame indices.
#[derive(Debug, Deserialize, Reflect)]
pub(crate) enum TrickfilmEntryKeyframes {
    /// You can specify the index of each frame seperately.
    KeyframesVec(Vec<usize>),
    /// Use this, if the animation frames of an animation have continuous indices.
    KeyframesRange(Range<usize>),
}

impl From<TrickfilmEntryKeyframes> for Vec<usize> {
    fn from(manifest: TrickfilmEntryKeyframes) -> Self {
        match manifest {
            TrickfilmEntryKeyframes::KeyframesVec(vec) => vec,
            TrickfilmEntryKeyframes::KeyframesRange(range) => range.collect(),
        }
    }
}

/// Representation of a loaded trickfilm file.
#[derive(Debug, Deserialize, Reflect)]
pub(crate) struct TrickfilmEntry {
    /// Keyframes of this animation
    keyframes: TrickfilmEntryKeyframes,
    /// Keyframe timestamps for this animation
    #[serde(default)]
    keyframe_timestamps: Option<Vec<f32>>,
    /// Duration of this animation
    duration: f32,
    /// User callbacks for this animation
    #[serde(default)]
    callbacks: Vec<String>,
}

/// File extension for spritesheet animation manifest files written in ron.
const FILE_EXTENSIONS: &[&str] = &["trickfilm.ron", "trickfilm"];

impl AssetLoader for Animation2DLoader {
    type Asset = AnimationClip2DSet;
    type Settings = ();
    type Error = Animation2DLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut dyn Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let trickfilm_entries = ron::de::from_bytes::<HashMap<String, TrickfilmEntry>>(&bytes)?;

        let animations: Result<HashMap<String, Handle<AnimationClip2D>>, AnimationClip2DError> =
            trickfilm_entries
                .into_iter()
                .map(|(name, entry)| {
                    let duration = entry.duration;
                    let keyframes: Vec<usize> = entry.keyframes.into();
                    let keyframe_timestamps = entry.keyframe_timestamps.unwrap_or(
                        (0..keyframes.len())
                            .map(|i| {
                                let i = i as f32 / keyframes.len() as f32;
                                i * duration
                            })
                            .collect(),
                    );
                    let callbacks = entry.callbacks;

                    let animation_clip = AnimationClip2D::new(
                        keyframe_timestamps,
                        keyframes,
                        duration,
                        callbacks,
                        &self.function_registry.read(),
                    )?;
                    Ok((
                        name.clone(),
                        load_context.add_labeled_asset(name, animation_clip),
                    ))
                })
                .collect();

        let animation_clip_2d_set = AnimationClip2DSet {
            animations: animations?,
        };
        Ok(animation_clip_2d_set)
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}
