//! This module contains the internals of the Animation2DLoader.
//!

use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    prelude::{AppTypeRegistry, FromWorld, World},
    reflect::TypeRegistryArc,
};
use ron::Deserializer;
use serde::de::DeserializeSeed;
use thiserror::Error;

use super::{AnimationClip2DError, AnimationClip2DSet, serde::AnimationClip2DSetDeserializer};

#[derive(Debug)]
pub(crate) struct Animation2DLoader {
    type_registry: TypeRegistryArc,
}

impl FromWorld for Animation2DLoader {
    fn from_world(world: &mut World) -> Self {
        let type_registry = world.resource::<AppTypeRegistry>();
        Self {
            type_registry: type_registry.0.clone(),
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
    #[error("AnimationClip2D has internal erro: {0}")]
    AnimationClip2DError(#[from] AnimationClip2DError),
}

/// File extension for spritesheet animation manifest files written in ron.
const FILE_EXTENSIONS: &[&str] = &["trickfilm.ron", "trickfilm"];

impl AssetLoader for Animation2DLoader {
    type Asset = AnimationClip2DSet;
    type Settings = ();
    type Error = Animation2DLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut deserializer = Deserializer::from_bytes(&bytes)?;
        let animationclip2dset_deserializer = AnimationClip2DSetDeserializer {
            type_registry: &self.type_registry.read(),
            load_context,
        };

        Ok(animationclip2dset_deserializer
            .deserialize(&mut deserializer)
            .map_err(|e| deserializer.span_error(e))?)
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}
