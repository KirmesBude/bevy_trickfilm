#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::prelude::{App, Plugin};

pub mod animation;
pub mod asset;

/// Adds support for 2d animation loading and playing.
pub struct Animation2DPlugin;

impl Plugin for Animation2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            asset::Animation2DLoaderPlugin,
            animation::AnimationPlayer2DPlugin,
        ));
    }
}

/// `use bevy_trickfilm::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::animation::AnimationEventAppExtension;
    pub use crate::animation::{
        AnimationPlayer2D, AnimationPlayer2DPlugin, AnimationPlayer2DSystemSet,
    };
    pub use crate::asset::{Animation2DLoaderPlugin, AnimationClip2D, AnimationClip2DSet};
    pub use crate::Animation2DPlugin;
}
