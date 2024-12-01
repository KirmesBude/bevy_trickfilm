//! This module implements everything necessary to support arbitrary events.
//!

use bevy::{app::Animation, prelude::*, reflect::GetTypeRegistration, utils::HashMap};

use crate::asset::AnimationClip2D;

use super::AnimationPlayer2D;

/// SystemSet to order animation playing and animation events
#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, SystemSet)]
pub(crate) struct AnimationEventSystemSet;

/// AnimationEvents are triggered by the animation system if registered as such with the App
pub trait AnimationEvent: Event + GetTypeRegistration + FromReflect + Clone {
    /// Implement this to be able to set the entity for a targeted event.
    /// Default implementation is a No-Op.
    fn set_target(&mut self, target: EventTarget) {
        let _ = target;
        /* Default implementation is empty for non-targeted events */
    }
}

/// Wrapper around entity to be used for EventTargets
#[derive(Debug, Clone, Copy, Deref, Reflect)]
pub struct EventTarget(pub Entity);

impl Default for EventTarget {
    fn default() -> Self {
        Self(Entity::PLACEHOLDER)
    }
}

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
    mut asset_events: EventReader<AssetEvent<AnimationClip2D>>,
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

// Collects events in a vector per entity for batching purposes
// Also calls AnimationEvent's set_target
fn collect_events<T: AnimationEvent>(
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: &AnimationEventCache<T>,
) -> HashMap<Entity, Vec<T>> {
    animation_players
        .iter()
        .map(|(entity, animation_player)| {
            let mut events: Vec<T> = Vec::with_capacity(0);
            if let Some(event_map) = cache.0.get(&animation_player.animation_clip().id()) {
                if animation_player.animation.last_frame != animation_player.animation.frame {
                    if let Some(animation_events) = event_map.get(&animation_player.frame()) {
                        events = animation_events.clone();
                        events
                            .iter_mut()
                            .for_each(|event| event.set_target(EventTarget(entity)));
                    }
                }
            }
            (entity, events)
        })
        .collect()
}

// Batch send events
fn send_animation_event<T: AnimationEvent>(
    mut event_writer: EventWriter<T>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventCache<T>>,
) {
    let entity_event_map = collect_events::<T>(animation_players, &cache);

    for (_, events) in entity_event_map {
        event_writer.send_batch(events);
    }
}

// Trigger events
fn trigger_animation_event<T: AnimationEvent>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    cache: Res<AnimationEventCache<T>>,
) {
    let entity_event_map = collect_events::<T>(animation_players, &cache);

    for (entity, events) in entity_event_map {
        for event in events {
            commands.trigger_targets(event, entity);
        }
    }
}

/// App extension trait to add animation_events/animation_triggers, which will schedule these sending/triggering systems for the specific type
pub trait AnimationEventAppExtension {
    /// Add event as buffered event.
    fn add_animation_event<T: AnimationEvent>(&mut self) -> &mut Self;

    /// Add event as observer.
    fn add_animation_trigger<T: AnimationEvent>(&mut self) -> &mut Self;
}

fn add_animation_cache<T: AnimationEvent>(app: &mut App) {
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
                .in_set(Animation)
                .in_set(AnimationEventSystemSet),
        );
    }

    app.register_type::<T>();
}

impl AnimationEventAppExtension for App {
    fn add_animation_event<T: AnimationEvent>(&mut self) -> &mut Self {
        add_animation_cache::<T>(self);

        self.add_event::<T>();
        self.add_systems(
            PostUpdate,
            send_animation_event::<T>
                .in_set(Animation)
                .in_set(AnimationEventSystemSet)
                .after(update_animation_event_cache::<T>),
        )
    }

    fn add_animation_trigger<T: AnimationEvent>(&mut self) -> &mut Self {
        add_animation_cache::<T>(self);

        // add_event is not necessary for observers
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<T>
                .in_set(Animation)
                .in_set(AnimationEventSystemSet)
                .after(update_animation_event_cache::<T>),
        )
    }
}
