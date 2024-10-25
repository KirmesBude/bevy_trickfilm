//! This module handles playing animations from an ['AnimationClip2D'](crate::asset::AnimationClip2D) asset using the ['AnimationPlayer2D'](crate::animation::AnimationPlayer2D) component.
//!

mod animation_spritesheet;
pub mod event;

use crate::prelude::AnimationClip2D;
use bevy::{
    animation::RepeatAnimation,
    app::PostUpdate,
    prelude::{App, Component, Handle, IntoSystemConfigs, Plugin, ReflectComponent, SystemSet},
    reflect::Reflect,
};

use self::animation_spritesheet::animation_player_spritesheet;

pub use event::{AnimationEvent, AnimationEventAppExtension};

/// A [`SystemSet`] to control where the animations are run
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimationPlayer2DSystemSet;

/// Adds support for spritesheet animation playing.
pub struct AnimationPlayer2DPlugin;

impl Plugin for AnimationPlayer2DPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AnimationPlayer2D>()
            .register_type::<PlayingAnimation2D>();
        app.add_systems(
            PostUpdate,
            animation_player_spritesheet.in_set(AnimationPlayer2DSystemSet),
        );
    }
}

#[derive(Reflect)]
pub(crate) struct PlayingAnimation2D {
    repeat: RepeatAnimation,
    speed: f32,
    elapsed: f32,
    duration: Option<f32>,
    pub(crate) last_frame: Option<usize>,
    frame: Option<usize>,
    seek_time: f32,
    animation_clip: Handle<AnimationClip2D>,
    completions: u32,
    completions_this_update: u32,
}

impl Default for PlayingAnimation2D {
    fn default() -> Self {
        Self {
            repeat: Default::default(),
            speed: 1.0,
            elapsed: 0.0,
            duration: None,
            last_frame: None,
            frame: None,
            seek_time: 0.0,
            animation_clip: Default::default(),
            completions: 0,
            completions_this_update: 0,
        }
    }
}

impl PlayingAnimation2D {
    /// Check if the playing animation has finished, according to [`RepeatAnimation`] repetition behavior.
    ///
    /// Note: An animation with [`RepeatAnimation::Forever`] will never finish.
    #[inline]
    pub fn finished(&self) -> bool {
        match self.repeat {
            RepeatAnimation::Forever => false,
            RepeatAnimation::Never => self.completions >= 1,
            RepeatAnimation::Count(n) => self.completions >= n,
        }
    }

    /// Check if the playing animation has just finished, according to [`RepeatAnimation`] repetition behavior.
    ///
    /// Note: An animation with [`RepeatAnimation::Forever`] will never finish.
    /// Note: This needs to be called in the [`bevy::prelude::Update`] schedule.
    /// Note: You should schedule it after [`AnimationPlayer2DSystemSet`] in [`PostUpdate`] to react to it on the same frame.
    #[inline]
    pub fn just_finished(&self) -> bool {
        self.finished() && self.just_finished_cycle()
    }

    /// Check if the playing animation has just finished a cycle.
    ///
    /// Note: This needs to be called in the [`bevy::prelude::Update`] schedule.
    /// Note: You should schedule it after [`AnimationPlayer2DSystemSet`] in [`PostUpdate`] to react to it on the same frame.
    #[inline]
    pub fn just_finished_cycle(&self) -> bool {
        self.completions_this_update > 0
    }

    /// Update the animation given the delta time and the duration of the clip being played.
    #[inline]
    fn update(&mut self, delta: f32, clip_duration: f32) {
        self.completions_this_update = 0;
        if self.finished() {
            return;
        }

        self.elapsed += delta;
        self.seek_time += delta * self.speed;

        // We determine the number of completions this update based on the seek_time and clip_duration.
        // For negative speeds where seek_time becomes negative, we need to consider that anything below 0.0 is already a completion.
        let quotient = (self.seek_time.abs() / clip_duration) as u32;
        self.completions_this_update = quotient + if self.seek_time < 0.0 { 1 } else { 0 };
        self.completions += self.completions_this_update;

        // Clamp the seek_time to [0.0, clip_duration].
        let modulo = self.seek_time.abs() % clip_duration;
        if self.seek_time >= clip_duration {
            self.seek_time = modulo;
        } else if self.seek_time < 0.0 {
            self.seek_time = clip_duration - modulo;
        }

        // If the animation is finished, we might not end up at the last frame if the delta step was too big.
        // Make sure that we have the last frame in that case
        if self.finished() {
            self.seek_time = clip_duration;
        }
    }

    /// Reset back to the initial state as if no time has elapsed.
    fn replay(&mut self) {
        self.completions_this_update = 0;
        self.completions = 0;
        self.elapsed = 0.0;
        self.seek_time = 0.0;
    }
}

/// Animation controls
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct AnimationPlayer2D {
    paused: bool,
    pub(crate) animation: PlayingAnimation2D,
}

impl AnimationPlayer2D {
    /// Start playing an animation, resetting state of the player.
    pub fn start(&mut self, handle: Handle<AnimationClip2D>) -> &mut Self {
        self.animation = PlayingAnimation2D {
            animation_clip: handle,
            ..Default::default()
        };
        self
    }

    /// Start playing an animation, resetting state of the player, unless the requested animation is already playing.
    pub fn play(&mut self, handle: Handle<AnimationClip2D>) -> &mut Self {
        if self.animation.animation_clip != handle || self.paused() {
            self.start(handle);
        }
        self
    }

    /// Handle to the animation clip being played.
    pub fn animation_clip(&self) -> &Handle<AnimationClip2D> {
        &self.animation.animation_clip
    }

    /// Check if the given animation clip is playing.
    pub fn clip_playing(&self, handle: &Handle<AnimationClip2D>) -> bool {
        self.animation_clip() == handle
    }

    /// Check if the playing animation has finished, according to [`RepeatAnimation`] repetition behavior.
    ///
    /// Note: An animation with [`RepeatAnimation::Forever`] will never finish.
    pub fn finished(&self) -> bool {
        self.animation.finished()
    }

    /// Check if the playing animation has just finished, according to [`RepeatAnimation`] repetition behavior.
    ///  
    /// Note: An animation with [`RepeatAnimation::Forever`] will never finish.  
    /// Note: This needs to be called in the [`bevy::prelude::Update`] schedule.  
    /// Note: You should schedule it after [`AnimationPlayer2DSystemSet`] in [`PostUpdate`] to react to it on the same frame.
    pub fn just_finished(&self) -> bool {
        self.animation.just_finished()
    }

    /// Check if the playing animation has just finished a cycle.
    ///  
    /// Note: This needs to be called in the [`bevy::prelude::Update`] schedule.  
    /// Note: You should schedule it after [`AnimationPlayer2DSystemSet`] in [`PostUpdate`] to react to it on the same frame.
    pub fn just_finished_cycle(&self) -> bool {
        self.animation.just_finished_cycle()
    }

    /// Number of times the animation has completed.
    pub fn completions(&self) -> u32 {
        self.animation.completions
    }

    /// How many completions the playing animation had this update.
    #[inline]
    pub fn completions_this_update(&self) -> u32 {
        self.animation.completions_this_update
    }

    /// Sets repeat to [`RepeatAnimation::Forever`].
    ///
    /// See also [`Self::set_repeat_mode`].
    pub fn repeat(&mut self) -> &mut Self {
        self.animation.repeat = RepeatAnimation::Forever;
        self
    }

    /// Set the repetition behaviour of the animation.
    pub fn set_repeat_mode(&mut self, repeat: RepeatAnimation) -> &mut Self {
        self.animation.repeat = repeat;
        self
    }

    /// Repetition behavior of the animation.
    pub fn repeat_mode(&self) -> RepeatAnimation {
        self.animation.repeat
    }

    /// Check if the animation is playing in reverse.
    pub fn reversed(&self) -> bool {
        self.animation.speed < 0.0
    }

    /// Pause the animation.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Unpause the animation
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Is the animation paused
    pub fn paused(&self) -> bool {
        self.paused
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
    ///
    /// Note: This is independent of speed.
    pub fn elapsed(&self) -> f32 {
        self.animation.elapsed
    }

    /// Duration of the playing animation if one is set, otherwise `None`
    ///
    /// Note: This is independent of speed.
    /// Note: Guaranteed to never return `Some(0.0)`.
    pub fn duration(&self) -> Option<f32> {
        self.animation.duration
    }

    /// Current frame of the animation
    ///
    /// This will be the same value as the index of the current animation in the spritesheet.
    pub fn frame(&self) -> usize {
        self.animation.frame.unwrap_or(0)
    }

    /// Seek time inside of the animation. Always within the range [0.0, clip_duration].
    pub fn seek_time(&self) -> f32 {
        self.animation.seek_time
    }

    /// Seek to a specific time in the animation.
    pub fn seek_to(&mut self, seek_time: f32) -> &mut Self {
        self.animation.seek_time = seek_time;
        self
    }

    /// Reset the animation to its initial state, as if no time has elapsed.
    pub fn replay(&mut self) {
        self.animation.replay();
    }
}
