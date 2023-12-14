//! This module handles playing animations from an ['AnimationClip2D'](crate::asset::AnimationClip2D) asset using the ['AnimationPlayer2D'](crate::animation::AnimationPlayer2D) component.
//!

mod animation_spritesheet;

use crate::prelude::AnimationClip2D;
use bevy::{
    prelude::{App, Component, Handle, Plugin, ReflectComponent, Update},
    reflect::Reflect,
};

use self::animation_spritesheet::animation_player_spritesheet;

/// Adds support for spritesheet animation playing.
pub struct AnimationPlayer2DPlugin;

impl Plugin for AnimationPlayer2DPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationPlayer2D>()
            .add_systems(Update, animation_player_spritesheet);
    }
}

#[derive(Reflect)]
struct PlayingAnimation2D {
    repeat: bool,
    finished: bool,
    speed: f32,
    elapsed: f32,
    animation_clip: Handle<AnimationClip2D>,
}

impl Default for PlayingAnimation2D {
    fn default() -> Self {
        Self {
            repeat: false,
            finished: false,
            speed: 1.0,
            elapsed: 0.0,
            animation_clip: Default::default(),
        }
    }
}

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationPlayer2D {
    paused: bool,
    animation: PlayingAnimation2D,
}

impl AnimationPlayer2D {
    /// Start playing an animation, resetting state of the player
    pub fn start(&mut self, handle: Handle<AnimationClip2D>) -> &mut Self {
        self.animation = PlayingAnimation2D {
            animation_clip: handle,
            ..Default::default()
        };
        self
    }

    /// Start playing an animation, resetting state of the player, unless the requested animation is already playing.
    pub fn play(&mut self, handle: Handle<AnimationClip2D>) -> &mut Self {
        if self.animation.animation_clip != handle || self.is_paused() {
            self.start(handle);
        }
        self
    }

    /// Set the animation to repeat
    pub fn repeat(&mut self) -> &mut Self {
        self.animation.repeat = true;
        self
    }

    /// Stop the animation from repeating
    pub fn stop_repeating(&mut self) -> &mut Self {
        self.animation.repeat = false;
        self
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Unpause the animation
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Is the animation paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Is the animation finished
    /// Always `false` for repeating animations.
    pub fn is_finished(&self) -> bool {
        self.animation.finished
    }

    /// Speed of the animation playback
    pub fn speed(&self) -> f32 {
        self.animation.speed
    }

    /// Set the speed of the animation playback
    pub fn set_speed(&mut self, speed: f32) -> &mut Self {
        self.animation.speed = speed;
        self
    }

    /// Time elapsed playing the animation
    pub fn elapsed(&self) -> f32 {
        self.animation.elapsed
    }

    /// Seek to a specific time in the animation
    pub fn set_elapsed(&mut self, elapsed: f32) -> &mut Self {
        self.animation.elapsed = elapsed;
        self
    }
}
