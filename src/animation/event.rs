//! This module animition event.
//!

use bevy::{prelude::*, reflect::GetTypeRegistration, utils::HashMap};

use crate::asset::AnimationClip2D;

use super::{
    animation_spritesheet::animation_player_spritesheet, AnimationPlayer2D,
    AnimationPlayer2DSystemSet,
};

/// AnimationEvents are triggered by the animation system if registered as such with the App.
pub trait AnimationEvent: Event + GetTypeRegistration + FromReflect {
    /// Implement this to be able to set the entity for a targeted event.
    /// Default implementation is a No-Op.
    fn set_entity(&mut self, entity: Entity) {
        let _ = entity;
        /* Default implementation is empty for non-targeted events */
    }
}

pub fn default_entity() -> Entity {
    Entity::PLACEHOLDER
}

fn collect_events<T: AnimationEvent>(
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: &Assets<AnimationClip2D>,
) -> HashMap<Entity, Vec<T>> {
    animation_players
        .iter()
        .map(|(entity, animation_player)| {
            let mut events = Vec::with_capacity(0);
            if let Some(animation_clip) = animation_clips.get(animation_player.animation_clip()) {
                // TODO: I need a better way to detect frame changes here
                // Get all events for this frame transition, if any
                if let Some(reflected_events) =
                    animation_clip.events().get(&animation_player.frame())
                {
                    events.reserve(reflected_events.len());
                    for reflected_event in reflected_events {
                        // TODO: Is this the most efficient way?
                        if let Some(mut event) = T::from_reflect(reflected_event.as_reflect()) {
                            event.set_entity(entity);
                            events.push(event);
                        }
                    }
                }
            }
            (entity, events)
        })
        .collect()
}

fn send_animation_event<T: AnimationEvent>(
    mut event_writer: EventWriter<T>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    let entity_event_map = collect_events::<T>(animation_players, &animation_clips);

    for (_, events) in entity_event_map {
        event_writer.send_batch(events);
    }
}

fn trigger_animation_event<T: AnimationEvent>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    let entity_event_map = collect_events::<T>(animation_players, &animation_clips);

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

impl AnimationEventAppExtension for App {
    fn add_animation_event<T: AnimationEvent>(&mut self) -> &mut Self {
        self.add_event::<T>().register_type::<T>();
        self.add_systems(
            PostUpdate,
            send_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }

    fn add_animation_trigger<T: AnimationEvent>(&mut self) -> &mut Self {
        self.register_type::<T>(); // add_event is not necessary for observers
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }
}
