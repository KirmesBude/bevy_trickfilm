use bevy::{
    asset::LoadContext,
    reflect::{
        serde::{ReflectDeserializer, TypeRegistrationDeserializer, TypedReflectDeserializer},
        Reflect, TypeRegistry,
    },
    utils::{HashMap, HashSet},
};
use serde::{
    de::{DeserializeSeed, Error, Visitor},
    Deserialize, Deserializer,
};

use super::{AnimationClip2D, AnimationClip2DSet, Keyframes};

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
        deserializer.deserialize_map(AnimationClip2DSetMapVisitor {
            type_registry: self.type_registry,
            load_context: self.load_context,
        })
    }
}

struct AnimationClip2DSetMapVisitor<'a, 'l> {
    pub type_registry: &'a TypeRegistry,
    pub load_context: &'a mut LoadContext<'l>,
}

impl<'a, 'l, 'de> Visitor<'de> for AnimationClip2DSetMapVisitor<'a, 'l> {
    type Value = AnimationClip2DSet;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of clips")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut value = HashMap::new();

        while let Some(name) = map.next_key::<String>()? {
            let clip = map.next_value_seed(AnimationClip2DDeserializer {
                type_registry: self.type_registry,
            })?;
            let asset = self.load_context.add_labeled_asset(name.clone(), clip);
            value.insert(name, asset);
        }

        Ok(AnimationClip2DSet { animations: value })
    }
}

#[derive(Deserialize)]
#[serde(field_identifier)]
enum AnimationClip2DField {
    #[serde(rename = "keyframes")]
    Keyframes,
    #[serde(rename = "keyframe_timestamps")]
    KeyframeTimestamps,
    #[serde(rename = "duration")]
    Duration,
    #[serde(rename = "events")]
    Events,
}

struct AnimationClip2DDeserializer<'a> {
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
        deserializer.deserialize_struct(
            "AnimationClip2D",
            &["keyframe_timestamps", "keyframes", "duration", "events"],
            AnimationClip2DVisitor {
                type_registry: self.type_registry,
            },
        )
    }
}

struct AnimationClip2DVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for AnimationClip2DVisitor<'a> {
    type Value = AnimationClip2D;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct of animation 2d clip")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut keyframes = None;
        let mut keyframe_timestamps = None;
        let mut duration = None;
        let mut events = None;

        while let Some(key) = map.next_key()? {
            match key {
                AnimationClip2DField::Keyframes => {
                    if keyframes.is_some() {
                        return Err(Error::duplicate_field("keyframes"));
                    }
                    keyframes = Some(map.next_value::<Keyframes>()?);
                }
                AnimationClip2DField::KeyframeTimestamps => {
                    if keyframe_timestamps.is_some() {
                        return Err(Error::duplicate_field("keyframe_timestamps"));
                    }
                    keyframe_timestamps = Some(map.next_value::<Vec<f32>>()?);
                }
                AnimationClip2DField::Duration => {
                    if duration.is_some() {
                        return Err(Error::duplicate_field("duration"));
                    }
                    duration = Some(map.next_value::<f32>()?);
                }
                AnimationClip2DField::Events => {
                    if events.is_some() {
                        return Err(Error::duplicate_field("events"));
                    }
                    events = Some(map.next_value_seed(AnimationEventsMapDeserializer {
                        type_registry: self.type_registry,
                    })?);
                }
            }
        }

        let keyframes = keyframes.ok_or_else(|| Error::missing_field("keyframes"))?;
        let duration = duration.ok_or_else(|| Error::missing_field("duration"))?;

        AnimationClip2D::new(keyframe_timestamps, keyframes, duration, events)
            .map_err(Error::custom)
    }
}

struct AnimationEventsMapDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for AnimationEventsMapDeserializer<'a> {
    type Value = HashMap<usize, Vec<Box<dyn Reflect>>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AnimationEventsMapVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct AnimationEventsMapVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for AnimationEventsMapVisitor<'a> {
    type Value = HashMap<usize, Vec<Box<dyn Reflect>>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of events")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut value = HashMap::new();

        while let Some(frame) = map.next_key::<usize>()? {
            let events = map.next_value_seed(AnimationEventsDeserializer {
                type_registry: self.type_registry,
            })?;
            value.insert(frame, events);
        }

        Ok(value)
    }
}

struct AnimationEventsDeserializer<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> DeserializeSeed<'de> for AnimationEventsDeserializer<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AnimationEventsVisitor {
            type_registry: self.type_registry,
        })
    }
}

struct AnimationEventsVisitor<'a> {
    pub type_registry: &'a TypeRegistry,
}

impl<'a, 'de> Visitor<'de> for AnimationEventsVisitor<'a> {
    type Value = Vec<Box<dyn Reflect>>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map of reflect types")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut dynamic_properties = Vec::new();
        while let Some(entity) =
            seq.next_element_seed(ReflectDeserializer::new(self.type_registry))?
        {
            dynamic_properties.push(entity);
        }

        Ok(dynamic_properties)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut added = HashSet::new();
        let mut entries = Vec::new();
        while let Some(registration) =
            map.next_key_seed(TypeRegistrationDeserializer::new(self.type_registry))?
        {
            if !added.insert(registration.type_id()) {
                return Err(Error::custom(format_args!(
                    "duplicate reflect type: `{}`",
                    registration.type_info().type_path(),
                )));
            }

            entries.push(map.next_value_seed(TypedReflectDeserializer::new(
                registration,
                self.type_registry,
            ))?);
        }

        Ok(entries)
    }
}
