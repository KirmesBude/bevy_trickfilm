use bevy::{asset::LoadContext, reflect::TypeRegistry};
use serde::{de::DeserializeSeed, Deserializer};

use super::{AnimationClip2D, AnimationClip2DSet};

pub struct AnimationClip2DSetDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'a>,
}

impl<'a, 'de> DeserializeSeed<'de> for AnimationClip2DSetDeserializer<'a> {
    type Value = AnimationClip2DSet;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}

pub struct AnimationClip2DDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for AnimationClip2DDeserializer<'a> {
    type Value = AnimationClip2D;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
