//! This module implements everything necessary to support arbitrary events.
//!

use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::{
    app::AnimationSystems,
    ecs::event::Trigger,
    platform::collections::HashMap,
    prelude::*,
    reflect::{GetTypeRegistration, Typed},
};

use crate::asset::AnimationClip2D;

use super::AnimationPlayer2D;

/// SystemSet to order animation playing and animation events
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct AnimationEventSystems;

#[derive(Debug, Resource)]
struct AnimationEventPayloadCache<T>(HashMap<AssetId<AnimationClip2D>, HashMap<usize, Vec<T>>>);

impl<T> Default for AnimationEventPayloadCache<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// This updates a cache resource for each Event added to the app
// That way when processing animation for event sending, we already have a vector of T instead of Box<dyn Reflect>, so we only iterate through the events that are actually relevant (can be from_reflected to T)
fn update_animation_event_payload_cache<T: AnimationEventPayload>(
    mut cache: ResMut<AnimationEventPayloadCache<T>>,
    mut asset_events: MessageReader<AssetEvent<AnimationClip2D>>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    for asset_event in asset_events.read() {
        match asset_event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                if let Some(clip) = animation_clips.get(*id) {
                    let inner_map = clip
                        .events()
                        .iter()
                        .map(|(frame, payloads)| {
                            (
                                *frame,
                                payloads
                                    .iter()
                                    .filter_map(|payload| {
                                        T::from_reflect(payload.as_partial_reflect())
                                    })
                                    .collect(),
                            )
                        })
                        .collect();
                    cache.0.entry(*id).insert(inner_map);
                } else {
                    debug!(
                        "Event {0:?} was triggered, but AssetId {1:?} does not yield an asset.",
                        asset_event, id
                    );
                }
            }
            AssetEvent::Removed { id } | AssetEvent::Unused { id } => {
                cache.0.remove(id);
            }
        }
    }
}

// Trigger events
fn trigger_animation_event<P: AnimationEventPayload, T: AnimationEventTrigger<P>>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventPayloadCache<P>>,
) where
    <T as AnimationEventTrigger<P>>::Trigger: std::default::Default,
{
    let events: Vec<AnimationEvent<P>> = animation_players
        .iter()
        .flat_map(|(entity, animation_player)| {
            let mut events: Vec<AnimationEvent<P>> = Vec::with_capacity(0);
            if let Some(payload_map) = cache.0.get(&animation_player.animation_clip().id())
                && animation_player.animation.last_frame != animation_player.animation.frame
                && let Some(animation_payloads) = payload_map.get(&animation_player.frame())
            {
                events = animation_payloads
                    .iter()
                    .map(|payload| AnimationEvent {
                        entity,
                        payload: payload.clone(),
                        trigger: PhantomData,
                    })
                    .collect();
            }
            events
        })
        .collect();

    for event in events {
        commands.trigger(event);
    }
}

// Batch write messages
fn write_animation_message<P: AnimationEventPayload>(
    mut event_writer: MessageWriter<AnimationMessage<P>>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventPayloadCache<P>>,
) {
    let messages: Vec<AnimationMessage<P>> = animation_players
        .iter()
        .flat_map(|(entity, animation_player)| {
            let mut events: Vec<AnimationMessage<P>> = Vec::with_capacity(0);
            if let Some(payload_map) = cache.0.get(&animation_player.animation_clip().id())
                && animation_player.animation.last_frame != animation_player.animation.frame
                && let Some(animation_payloads) = payload_map.get(&animation_player.frame())
            {
                events = animation_payloads
                    .iter()
                    .map(|payload| AnimationMessage {
                        entity,
                        payload: payload.clone(),
                    })
                    .collect();
            }
            events
        })
        .collect();

    event_writer.write_batch(messages);
}

/// App extension trait to add AnimationEvents, which will schedule the triggering systems for the specific type
pub trait AnimationEventAppExtension {
    /// Add event
    fn add_animation_event<P: AnimationEventPayload>(&mut self) -> &mut Self;

    /// Add event
    fn add_animation_event_with_trigger<P: AnimationEventPayload, T: AnimationEventTrigger<P>>(
        &mut self,
    ) -> &mut Self
    where
        <T as AnimationEventTrigger<P>>::Trigger: std::default::Default;
}

impl AnimationEventAppExtension for App {
    fn add_animation_event<P: AnimationEventPayload>(&mut self) -> &mut Self {
        self.add_animation_event_with_trigger::<P, DefaultAnimationEventTrigger>()
    }

    fn add_animation_event_with_trigger<P: AnimationEventPayload, T: AnimationEventTrigger<P>>(
        &mut self,
    ) -> &mut Self
    where
        <T as AnimationEventTrigger<P>>::Trigger: std::default::Default,
    {
        add_animation_event_payload_cache::<P>(self);

        self.register_type::<P>();
        self.register_type::<AnimationEvent<P, T>>();

        // add_event is not necessary for observers
        // TODO: Maybe putting this here defeats the purpose, but how to get it as close as possible to the actual animation?
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<P, T>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems)
                .after(update_animation_event_payload_cache::<P>),
        )
    }
}

/// App extension trait to add AnimationMessages, which will schedule the writing systems for the specific type
pub trait AnimationMessageAppExtension {
    /// Add event
    fn add_animation_message<P: AnimationEventPayload>(&mut self) -> &mut Self;
}

impl AnimationMessageAppExtension for App {
    fn add_animation_message<P: AnimationEventPayload>(&mut self) -> &mut Self {
        add_animation_event_payload_cache::<P>(self);

        self.register_type::<P>();
        self.register_type::<AnimationMessage<P>>();

        self.add_message::<AnimationMessage<P>>();
        self.add_systems(
            PostUpdate,
            write_animation_message::<P>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems)
                .after(update_animation_event_payload_cache::<P>),
        )
    }
}

fn add_animation_event_payload_cache<P: AnimationEventPayload>(app: &mut App) {
    // Handle caching, if it does not already exist
    if app
        .world()
        .get_resource::<AnimationEventPayloadCache<P>>()
        .is_none()
    {
        app.init_resource::<AnimationEventPayloadCache<P>>();
        app.add_systems(
            PostUpdate,
            update_animation_event_payload_cache::<P>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems),
        );
    }
}

/// TODO Trigger
pub trait AnimationEventTrigger<P: AnimationEventPayload>: Sized + 'static + TypePath {
    /// Trigger that is passed to AnimationEvent.
    type Trigger: Trigger<AnimationEvent<P, Self>>;
}

/// TODO DefaultTrigger
#[derive(Debug, Default, Copy, Clone, Reflect)]
pub struct DefaultAnimationEventTrigger;

impl<P: AnimationEventPayload> AnimationEventTrigger<P> for DefaultAnimationEventTrigger {
    type Trigger = bevy::ecs::event::GlobalTrigger;
}

/// TODO: Payloads
pub trait AnimationEventPayload: GetTypeRegistration + FromReflect + Typed + Clone {}

/// AnimationEvent typed by your payload
#[derive(Clone, Reflect)]
pub struct AnimationEvent<
    P: AnimationEventPayload,
    T: AnimationEventTrigger<P> = DefaultAnimationEventTrigger,
> {
    /// Entity that has the AnimationPlayer2D
    pub entity: Entity,
    /// Payload with your arbitrary data
    pub payload: P,
    #[reflect(ignore)]
    trigger: PhantomData<fn() -> T>,
}

impl<P: AnimationEventPayload, T: AnimationEventTrigger<P>> Event for AnimationEvent<P, T> {
    type Trigger<'a> = T::Trigger;
}

impl<P: AnimationEventPayload, T: AnimationEventTrigger<P>> EntityEvent for AnimationEvent<P, T> {
    fn event_target(&self) -> Entity {
        self.entity
    }

    fn event_target_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl<P: AnimationEventPayload + Debug, T: AnimationEventTrigger<P>> Debug for AnimationEvent<P, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationEvent")
            .field("entity", &self.entity)
            .field("payload", &self.payload)
            .finish()
    }
}

/// AnimationMessage typed by your payload
#[derive(Debug, Reflect)]
pub struct AnimationMessage<P: AnimationEventPayload> {
    /// Entity that has the AnimationPlayer2D
    pub entity: Entity,
    /// Payload with your arbitrary data
    pub payload: P,
}

impl<P: AnimationEventPayload> Message for AnimationMessage<P> {}
