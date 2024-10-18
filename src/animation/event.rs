use bevy::prelude::*;

use crate::asset::AnimationClip2D;

use super::{
    animation_spritesheet::animation_player_spritesheet, AnimationPlayer2D,
    AnimationPlayer2DSystemSet,
};

/// AnimationEvents are triggered by the animation system if registered as such with the App.
pub trait AnimationEvent: Event + FromReflect {
    /// Implement this to be able to set the entity for a targeted event.
    /// Default implementation is a No-Op.
    fn set_entity(&mut self, entity: Entity) {
        let _ = entity;
        /* Default implementation is empty for non-targeted events */
    }
}

// Define generic trigger and event systems
pub fn send_animation_event<T: AnimationEvent>(
    mut event_writer: EventWriter<T>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    for (entity, animation_player) in &animation_players {
        if let Some(animation_clip) = animation_clips.get(animation_player.animation_clip()) {
            if let Some(reflected_events) = animation_clip.events().get(&animation_player.frame()) {
                for reflected_event in reflected_events {
                    // TODO: Patch in entity somehow
                    if let Some(mut event) = T::from_reflect(reflected_event.as_reflect()) {
                        // TODO: batch
                        event.set_entity(entity);
                        event_writer.send(event);
                    }
                }
            }
        }
    }
}

// TODO: generalize the query stuff
pub fn trigger_animation_event<T: AnimationEvent>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    for (entity, animation_player) in &animation_players {
        if let Some(animation_clip) = animation_clips.get(animation_player.animation_clip()) {
            if let Some(reflected_events) = animation_clip.events().get(&animation_player.frame()) {
                for reflected_event in reflected_events {
                    // TODO: Patch in entity somehow
                    if let Some(mut event) = T::from_reflect(reflected_event.as_reflect()) {
                        // TODO: batch
                        event.set_entity(entity);
                        commands.trigger(event);
                    }
                }
            }
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
        self.add_systems(
            PostUpdate,
            send_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }

    fn add_animation_trigger<T: AnimationEvent>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }
}
