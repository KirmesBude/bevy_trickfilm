use bevy::{asset::LoadContext, reflect::TypeRegistry, utils::HashMap};
use serde::{de::DeserializeSeed, Deserializer};

use super::{AnimationClip2D, AnimationClip2DSet};

pub struct AnimationClip2DSetDeserializer<'a, 'l> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'l>,
}

impl<'a, 'l, 'de> DeserializeSeed<'de> for AnimationClip2DSetDeserializer<'a, 'l> {
    type Value = AnimationClip2DSet;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        /* Essentially a map */
        /* Deserialize a HashMap with String keys -> delegate to AnimationClip2DDeserializer for values */
        /* -> use load_context to get Handles from it */
        let hash_map: HashMap<String, AnimationClip2D> = HashMap::new();

        let animation2dclip_deserializer = AnimationClip2DDeserializer {
            type_registry: &self.type_registry,
        };
        let _ = animation2dclip_deserializer.deserialize(deserializer);

        Ok(AnimationClip2DSet {
            animations: hash_map
                .into_iter()
                .map(|(name, clip)| {
                    (
                        name.clone(),
                        self.load_context.add_labeled_asset(name, clip),
                    )
                })
                .collect(),
        })
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
        /* Essentially a struct */
        /* mandatory keyframes of type Keyframes */
        /* optional keyframe_timestamps of type Vec<usize> */
        /* mandatory duration of type f32 */
        /* optional events of type Box<dyn Reflect> -> use type_registry to reflect the information */

        todo!()
    }
}
