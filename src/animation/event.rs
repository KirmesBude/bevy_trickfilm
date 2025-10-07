//! This module implements everything necessary to support arbitrary events.
//!

use bevy::{
    app::AnimationSystems, platform::collections::HashMap, prelude::*, reflect::GetTypeRegistration,
};

use crate::asset::AnimationClip2D;

use super::AnimationPlayer2D;

/// SystemSet to order animation playing and animation events
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct AnimationEventSystems;

/// Deprecated alias for [`AnimationEventSystemSet`].
#[deprecated(since = "0.12.0", note = "Renamed to `AnimationEventSystems`.")]
pub type AnimationEventSystemSet = AnimationEventSystems;

/// AnimationEvents are triggered by the animation system if registered as such with the App
pub trait AnimationEvent: Event + GetTypeRegistration + FromReflect + Clone {}

/// AnimationEntityEvents are triggered by the animation system if registered as such with the App
pub trait AnimationEntityEvent: EntityEvent + GetTypeRegistration + FromReflect + Clone {
    fn set_entity(&mut self, entity: Entity);
}

/// AnimationMessages are written by the animation system if registered as such with the App
pub trait AnimationMessage: Message + GetTypeRegistration + FromReflect + Clone {}

#[derive(Debug, Resource)]
struct AnimationEventCache<T>(HashMap<AssetId<AnimationClip2D>, HashMap<usize, Vec<T>>>);

impl<T> Default for AnimationEventCache<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// This updates a cache resource for each Event added to the app
// That way when processing animation for event sending, we already have a vector of T instead of Box<dyn Reflect>, so we only iterate through the events that are actually relevant (can be from_reflected to T)
fn update_animation_event_cache<T: FromReflect>(
    mut cache: ResMut<AnimationEventCache<T>>,
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
                        .map(|(frame, events)| {
                            (
                                *frame,
                                events
                                    .iter()
                                    .filter_map(|event| T::from_reflect(event.as_partial_reflect()))
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

// Collects events in a vector for batching purposes
fn collect_events<T: AnimationEvent>(
    animation_players: Query<&AnimationPlayer2D>,
    cache: &AnimationEventCache<T>,
) -> Vec<T> {
    animation_players
        .iter()
        .map(|animation_player| {
            let mut events: Vec<T> = Vec::with_capacity(0);
            if let Some(event_map) = cache.0.get(&animation_player.animation_clip().id()) {
                if animation_player.animation.last_frame != animation_player.animation.frame {
                    if let Some(animation_events) = event_map.get(&animation_player.frame()) {
                        events = animation_events.clone();
                    }
                }
            }
            events
        })
        .flatten()
        .collect()
}

// Collects events in a vector per entity for batching purposes
// Also calls AnimationEvent's set_target
fn collect_entity_events<T: AnimationEntityEvent>(
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: &AnimationEventCache<T>,
) -> Vec<T> {
    animation_players
        .iter()
        .map(|(entity, animation_player)| {
            let mut events: Vec<T> = Vec::with_capacity(0);
            if let Some(event_map) = cache.0.get(&animation_player.animation_clip().id()) {
                if animation_player.animation.last_frame != animation_player.animation.frame {
                    if let Some(animation_events) = event_map.get(&animation_player.frame()) {
                        events = animation_events.clone();
                        events.iter_mut().for_each(|event| event.set_entity(entity));
                    }
                }
            }
            events
        })
        .flatten()
        .collect()
}

// Collects events in a vector for batching purposes
fn collect_messages<T: AnimationMessage>(
    animation_players: Query<&AnimationPlayer2D>,
    cache: &AnimationEventCache<T>,
) -> Vec<T> {
    animation_players
        .iter()
        .map(|animation_player| {
            let mut events: Vec<T> = Vec::with_capacity(0);
            if let Some(event_map) = cache.0.get(&animation_player.animation_clip().id()) {
                if animation_player.animation.last_frame != animation_player.animation.frame {
                    if let Some(animation_events) = event_map.get(&animation_player.frame()) {
                        events = animation_events.clone();
                    }
                }
            }
            events
        })
        .flatten()
        .collect()
}

// Trigger events
fn trigger_animation_event<T: AnimationEvent>(
    mut commands: Commands,
    animation_players: Query<&AnimationPlayer2D>,
    cache: Res<AnimationEventCache<T>>,
) {
    let events = collect_events::<T>(animation_players, &cache);

    for event in events {
        commands.trigger(event);
    }
}

// Trigger entity events
fn trigger_animation_entity_event<T: AnimationEntityEvent>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventCache<T>>,
) {
    let events = collect_entity_events::<T>(animation_players, &cache);

    for event in events {
        commands.trigger(event);
    }
}

// Batch write messages
fn write_animation_message<T: AnimationMessage>(
    mut event_writer: MessageWriter<T>,
    animation_players: Query<&AnimationPlayer2D>,
    cache: Res<AnimationEventCache<T>>,
) {
    let messages = collect_messages::<T>(animation_players, &cache);

    event_writer.write_batch(messages);
}

/// App extension trait to add AnimationEvents, which will schedule the triggering systems for the specific type
pub trait AnimationEventAppExtension {
    /// Add event
    fn add_animation_event<T: AnimationEvent>(&mut self) -> &mut Self;
}

impl AnimationEventAppExtension for App {
    fn add_animation_event<T: AnimationEvent>(&mut self) -> &mut Self {
        add_animation_cache::<T>(self);

        // add_event is not necessary for observers
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<T>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems)
                .after(update_animation_event_cache::<T>),
        )
    }
}

/// App extension trait to add AnimationEntityEvents, which will schedule the triggering systems for the specific type
pub trait AnimationEntityEventAppExtension {
    /// Add event
    fn add_animation_entity_event<T: AnimationEntityEvent>(&mut self) -> &mut Self;
}

impl AnimationEntityEventAppExtension for App {
    fn add_animation_entity_event<T: AnimationEntityEvent>(&mut self) -> &mut Self {
        add_animation_cache::<T>(self);

        self.add_systems(
            PostUpdate,
            trigger_animation_entity_event::<T>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems)
                .after(update_animation_event_cache::<T>),
        )
    }
}

/// App extension trait to add AnimationMessages, which will schedule the writing systems for the specific type
pub trait AnimationMessageAppExtension {
    /// Add event
    fn add_animation_message<T: AnimationMessage>(&mut self) -> &mut Self;
}

impl AnimationMessageAppExtension for App {
    fn add_animation_message<T: AnimationMessage>(&mut self) -> &mut Self {
        add_animation_cache::<T>(self);

        self.add_message::<T>();
        self.add_systems(
            PostUpdate,
            write_animation_message::<T>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems)
                .after(update_animation_event_cache::<T>),
        )
    }
}

fn add_animation_cache<T: Send + Sync + GetTypeRegistration + FromReflect>(app: &mut App) {
    // Handle caching, if it does not already exist
    if app
        .world()
        .get_resource::<AnimationEventCache<T>>()
        .is_none()
    {
        app.init_resource::<AnimationEventCache<T>>();
        app.add_systems(
            PostUpdate,
            update_animation_event_cache::<T>
                .in_set(AnimationSystems)
                .in_set(AnimationEventSystems),
        );
    }

    app.register_type::<T>();
}
