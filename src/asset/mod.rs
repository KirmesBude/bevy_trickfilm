//! This module defines all assets for 2D Animations.
//! This crate allows you to directly load an [`AnimationClip2DSet`] and/or [`AnimationClip2D`] from a manifest file.
//! Assets with the 'trickfilm' extension can be loaded just like any other asset via the [`AssetServer`](bevy::asset::AssetServer)
//! and will yield an [`AnimationClip2DSet`] [`Handle`] (or an [`AnimationClip2D`] [`Handle`] directly via labeled assets).
//!

use std::cmp::Ordering;
use std::ops::Range;

use ::serde::Deserialize;
use asset_loader::{TrickfilmEntry, TrickfilmEntryKeyframes};
use bevy::{
    prelude::{App, Asset, AssetApp, Handle, Plugin},
    reflect::{Reflect, TypePath},
    utils::HashMap,
};
use thiserror::Error;

use self::asset_loader::Animation2DLoader;

pub mod asset_loader;
mod serde;

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct Animation2DLoaderPlugin;

impl Plugin for Animation2DLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            /*
            .register_type::<AnimationClip2D>()
            .register_type::<AnimationClip2DSet>()
            */
            .register_type::<TrickfilmEntryKeyframes>()
            .register_type::<TrickfilmEntry>();
        app.init_asset::<AnimationClip2D>()
            .init_asset::<AnimationClip2DSet>()
            .init_asset_loader::<Animation2DLoader>();
    }
}

#[derive(Debug, Deserialize)]
pub enum Keyframes {
    KeyframesVec(Vec<usize>),
    KeyframesRange(Range<usize>),
}

impl From<Keyframes> for Vec<usize> {
    fn from(keyframes: Keyframes) -> Self {
        match keyframes {
            Keyframes::KeyframesVec(vec) => vec,
            Keyframes::KeyframesRange(range) => range.collect(),
        }
    }
}

impl Keyframes {
    pub fn len(&self) -> usize {
        match self {
            Keyframes::KeyframesVec(vec) => vec.len(),
            Keyframes::KeyframesRange(range) => range.len(),
        }
    }

    pub fn get(&self, index: usize) -> Option<usize> {
        match self {
            Keyframes::KeyframesVec(vec) => Some(vec[index]),
            Keyframes::KeyframesRange(range) => Some(range.start + index),
        }
    }
}

/// AnimationClip for a 2D animation.
#[derive(Asset, TypePath, Debug)]
pub struct AnimationClip2D {
    /// Timestamps for each keyframe in seconds.
    keyframe_timestamps: Vec<f32>,
    /// An ordered list of incides of the TextureAtlas or Images that represent the frames of this animation.
    keyframes: Keyframes,
    /// Total duration of this animation clip in seconds.
    duration: f32,
    events: HashMap<usize, Vec<Box<dyn Reflect>>>,
}

/// Possible errors that can be produced by [`AnimationClip2D`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum AnimationClip2DError {
    /// Error that occurs, if the size of keyframes and keyframe_timestamp does not match.
    #[error("Size of keyframes and keyframe_timestamps does not match: {0} and {1}")]
    SizeMismatch(usize, usize),
    /// Error that occurs, if no keyframes are provided.
    #[error("Animation clip is empty, because the size of keyframes is 0")]
    Empty(),
    /// Error that occurs, if duration is not sufficient to play all keyframes.
    #[error("Duration of {0} is insufficient to display last keyframe at {1}")]
    InsufficientDuration(f32, f32),
    /// Error that occurs, if an events references a frame outside the frame range.
    #[error("Frame {0} for this animation clip, because it only has {1} frames")]
    InvalidFrame(usize, usize),
}

impl AnimationClip2D {
    /// Creates a valid [`AnimationClip2D`]
    pub fn new(
        keyframe_timestamps: Vec<f32>,
        keyframes: Keyframes,
        duration: f32,
        events: HashMap<usize, Vec<Box<dyn Reflect>>>,
    ) -> Result<Self, AnimationClip2DError> {
        let keyframe_timestamps_len = keyframe_timestamps.len();
        let keyframes_len = keyframes.len();
        if keyframe_timestamps_len != keyframes_len {
            return Err(AnimationClip2DError::SizeMismatch(
                keyframe_timestamps_len,
                keyframes_len,
            ));
        }

        if keyframe_timestamps_len == 0 {
            return Err(AnimationClip2DError::Empty());
        }

        let keyframe_timestamps_max = keyframe_timestamps
            .iter()
            .max_by(|x, y| {
                x.partial_cmp(y)
                    .expect("Keyframe timestamps contain elements, that are not comparable.")
            })
            .expect("Already covered by AnimationClip2DError::Empty().");
        if let Some(Ordering::Greater) = keyframe_timestamps_max.partial_cmp(&duration) {
            return Err(AnimationClip2DError::InsufficientDuration(
                *keyframe_timestamps_max,
                duration,
            ));
        }

        let max_event_frame = events.keys().max().cloned().unwrap_or(0);
        if max_event_frame > keyframes_len {
            return Err(AnimationClip2DError::InvalidFrame(
                max_event_frame,
                keyframes_len,
            ));
        }

        Ok(Self {
            keyframe_timestamps,
            keyframes,
            duration,
            events,
        })
    }

    /// Timestamps for each keyframe in seconds.
    #[inline]
    pub fn keyframe_timestamps(&self) -> &[f32] {
        &self.keyframe_timestamps
    }

    /// Ordered list of keyframes for this animation.
    #[inline]
    pub fn keyframes(&self) -> &Keyframes {
        &self.keyframes
    }

    /// Total duration of this animation clip in seconds.
    #[inline]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// All reflected events for this animation clip identified by their associated frame.
    #[inline]
    pub fn events(&self) -> &HashMap<usize, Vec<Box<dyn Reflect>>> {
        &self.events
    }
}

/// Set(Map) of AnimationClips for a 2D animation.
#[derive(Asset, TypePath, Debug)]
pub struct AnimationClip2DSet {
    /// Named animations loaded from the trickfilm file.
    pub animations: HashMap<String, Handle<AnimationClip2D>>,
}
