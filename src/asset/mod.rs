//! This module defines all assets for 2D Animations.
//!

use bevy::{
    prelude::{App, Asset, AssetApp, Handle, Plugin},
    reflect::TypePath,
    utils::HashMap,
};
use thiserror::Error;

use self::asset_loader::Animation2DLoader;

pub mod asset_loader;

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct Animation2DLoaderPlugin;

impl Plugin for Animation2DLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AnimationClip2D>()
            .init_asset::<Trickfilm>()
            .init_asset_loader::<Animation2DLoader>();
    }
}

/// AnimationClip for a 2D animation.
#[derive(Asset, Debug, TypePath)]
pub struct AnimationClip2D {
    /// Timestamps for each keyframe in seconds.
    keyframe_timestamps: Vec<f32>,
    /// An ordered list of incides of the TextureAtlas or Images that represent the frames of this animation.
    keyframes: Vec<usize>,
    /// Total duration of this animation clip in seconds.
    duration: f32,
}

/// Possible errors that can be produced by [`AnimationClip2D`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AnimationClip2DError {
    #[error("Size of keyframes and keyframe_timestamps does not match: {0} and {1}")]
    SizeMismatch(usize, usize),
    /* TODO: Handle duration less than max keyframe_timestamps */
}

impl AnimationClip2D {
    pub fn new(
        keyframe_timestamps: Vec<f32>,
        keyframes: Vec<usize>,
        duration: f32,
    ) -> Result<Self, AnimationClip2DError> {
        let keyframe_timestamps_len = keyframe_timestamps.len();
        let keyframes_len = keyframes.len();
        if keyframe_timestamps_len == keyframes_len {
            Ok(Self {
                keyframe_timestamps,
                keyframes,
                duration,
            })
        } else {
            Err(AnimationClip2DError::SizeMismatch(
                keyframe_timestamps_len,
                keyframes_len,
            ))
        }
    }

    /// Timestamps for each keyframe in seconds.
    #[inline]
    pub fn keyframe_timestamps(&self) -> &[f32] {
        &self.keyframe_timestamps
    }

    /// Ordered list of [`Keyframes2D`] elements for this animation.
    #[inline]
    pub fn keyframes(&self) -> &[usize] {
        &self.keyframes
    }

    /// Total duration of this animation clip in seconds.
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }
}

/// Representation of a loaded trickfilm file.
#[derive(Asset, Debug, TypePath)]
pub struct Trickfilm {
    /// Named animations loaded from the trickfilm file.
    pub animations: HashMap<String, Handle<AnimationClip2D>>,
}
