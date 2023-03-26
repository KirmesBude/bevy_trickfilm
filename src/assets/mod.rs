//! This module defines all Assets SpriteSheetAnimationSet from a manifest file.
//!

use bevy::{
    prelude::{AddAsset, App, Handle, Image, Plugin},
    reflect::TypeUuid,
    sprite::TextureAtlas,
    utils::HashMap,
};

use self::asset_loader::Animation2DLoader;

pub mod asset_loader;

/// Adds support for spritesheet animation manifest files loading to the app.
pub struct Animation2DLoaderPlugin;

impl Plugin for Animation2DLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<AnimationClip2D>()
            .add_asset::<AnimationClipSet2D>()
            .init_asset_loader::<Animation2DLoader>();
    }
}

/// Declaration of the deserialized variant for the animation keyframes.
#[derive(Debug, Clone)]
pub enum Keyframes2D {
    /// For Spritesheet animations
    SpriteSheet(Handle<TextureAtlas>, Vec<usize>),
    /// For Sprite animations
    Sprite(Vec<Handle<Image>>),
}

impl Default for Keyframes2D {
    fn default() -> Self {
        Self::Sprite(vec![])
    }
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "9403342c-8c4e-495e-85ef-3e9cd12ffea5"]
pub struct AnimationClip2D {
    /// Timestamp for each keyframe.
    pub keyframe_timestamps: Vec<f32>,
    /// An ordered list of incides of the TextureAtlas or Images that represent the frames of this animation.
    pub keyframes: Keyframes2D,
    /// Total duration of this animation clip.
    pub duration: f32,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "ec942212-87dc-4ee4-8300-1e160a389c37"]
pub struct AnimationClipSet2D {
    /// Optional name of this animation set.
    pub name: Option<String>,
    /// A map of all animations in this set, identified by their names.
    pub animations: HashMap<String, Handle<AnimationClip2D>>,
}
