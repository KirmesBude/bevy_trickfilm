use bevy::prelude::*;

use crate::asset::AnimationClip2D;

use super::{
    animation_spritesheet::animation_player_spritesheet, AnimationPlayer2D,
    AnimationPlayer2DSystemSet,
};

// Trait
pub trait AnimationEvent: Event {
    fn from_reflect(entity: Entity, reflect: &dyn Reflect) -> Self;
}

// Define generic trigger and event systems
pub fn send_animation_event<T: Event + AnimationEvent>(
    mut event_writer: EventWriter<T>,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    for (entity, animation_player) in &animation_players {
        if let Some(animation_clip) = animation_clips.get(animation_player.animation_clip()) {
            // TODO: Get "Reflect" data from event field on AnimationClip2D
            let reflect = ().as_reflect();

            // TODO: batch
            event_writer.send(T::from_reflect(entity, reflect));
        }
    }
}

// TODO: generalize the query stuff
pub fn trigger_animation_event<T: Event + AnimationEvent>(
    mut commands: Commands,
    animation_players: Query<(Entity, &AnimationPlayer2D)>,
    animation_clips: Res<Assets<AnimationClip2D>>,
) {
    for (entity, animation_player) in &animation_players {
        if let Some(animation_clip) = animation_clips.get(animation_player.animation_clip()) {
            // TODO: Get "Reflect" data from event field on AnimationClip2D
            let reflect = ().as_reflect();

            // TODO: batch
            commands.trigger(T::from_reflect(entity, reflect));
        }
    }
}

// Add extension to app to add animation_events/animation_triggers, which will schedule these systems for the specific type

pub trait AnimationEventAppExtension {
    fn add_animation_event<T: Event + AnimationEvent>(&mut self) -> &mut Self;

    fn add_animation_trigger<T: Event + AnimationEvent>(&mut self) -> &mut Self;
}

impl AnimationEventAppExtension for App {
    fn add_animation_event<T: Event + AnimationEvent>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            send_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }

    fn add_animation_trigger<T: Event + AnimationEvent>(&mut self) -> &mut Self {
        self.add_systems(
            PostUpdate,
            trigger_animation_event::<T>
                .in_set(AnimationPlayer2DSystemSet)
                .after(animation_player_spritesheet),
        )
    }
}
