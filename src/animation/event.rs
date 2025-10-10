//! This module implements everything necessary to support arbitrary events.
//!

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

// Collect events with payload
fn collect_events<P: AnimationEventPayload>(
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: &AnimationEventPayloadCache<P>,
) -> Vec<AnimationEvent<P>> {
    animation_players
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
                    })
                    .collect();
            }
            events
        })
        .collect()
}

// Trigger events
fn trigger_animation_event<P: AnimationEventPayload>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventPayloadCache<P>>,
) where
    <P as AnimationEventPayload>::Trigger: std::default::Default,
{
    let events = collect_events::<P>(animation_players, &cache);

    for event in events {
        commands.trigger(event);
    }
}

// Batch write messages
fn write_animation_message<P: AnimationEventPayload>(
    mut event_writer: MessageWriter<AnimationEvent<P>>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventPayloadCache<P>>,
) {
    let messages = collect_events::<P>(animation_players, &cache);

    event_writer.write_batch(messages);
}

/// App extension trait to add AnimationEvents, which will schedule the triggering systems for the specific type
pub trait AnimationEventAppExtension {
    /// Add event
    fn add_animation_event<P: AnimationEventPayload>(&mut self) -> &mut Self
    where
        <P as AnimationEventPayload>::Trigger: std::default::Default;
}

impl AnimationEventAppExtension for App {
    fn add_animation_event<P: AnimationEventPayload>(&mut self) -> &mut Self
    where
        <P as AnimationEventPayload>::Trigger: std::default::Default,
    {
        add_animation_event_payload_cache::<P>(self);

        // add_event is not necessary for observers
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<P>
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

        self.add_message::<AnimationEvent<P>>();
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

    app.register_type::<P>();
    app.register_type::<AnimationEvent<P>>();
}

/// TODO: Payloads
pub trait AnimationEventPayload: GetTypeRegistration + FromReflect + Typed + Clone {
    /// Trigger that is passed to AnimationEvent.
    type Trigger: Trigger<AnimationEvent<Self>>;
}

/// AnimationEvent typed by your AnimationEventPayload
#[derive(Debug, Reflect)]
pub struct AnimationEvent<P: AnimationEventPayload> {
    /// Entity that has the AnimationPlayer2D
    pub entity: Entity,
    /// Payload with your arbitrary data
    pub payload: P,
}

impl<P: AnimationEventPayload> Event for AnimationEvent<P> {
    type Trigger<'a> = P::Trigger;
}

impl<P: AnimationEventPayload> EntityEvent for AnimationEvent<P> {
    fn event_target(&self) -> Entity {
        self.entity
    }

    fn event_target_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}

impl<P: AnimationEventPayload> Message for AnimationEvent<P> {}
