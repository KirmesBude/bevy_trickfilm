//! This module handles playing animations from an ['SpriteSheetAnimationSet'](crate::asset_loader::SpriteSheetAnimationSet) Asset.
//!
//! `bevy_trickfilm::animation` introduces a [`SpriteSheetAnimationPlayer`](crate::animation::SpriteSheetAnimationPlayer) component.
//! The component supports playing and stopping animations.
//!
//! ```edition2021
//! # use bevy_trickfilm::prelude::*;
//! # use bevy::prelude::*;
//! #
//! ...
//!
//! fn kick(mut animation_players: Query<&mut SpriteSheetAnimationPlayer, With<Controlled>>, keys: Res<Input<KeyCode>>) {
//!     if keys.just_pressed(KeyCode::Space) {
//!         for mut animation_player in &mut animation_players {
//!             animation_player.play(String::from("kick"));
//!         }
//!     }
//! }
//!
//! ```

use crate::{assets::Keyframes2D, prelude::AnimationClip2D};
use bevy::{
    prelude::{
        App, Assets, Changed, Component, DetectChanges, Handle, IntoSystemDescriptor, Mut, Plugin,
        Query, ReflectComponent, Res,
    },
    reflect::Reflect,
    sprite::{TextureAtlas, TextureAtlasSprite},
    time::Time,
};

/// Adds support for spritesheet animation playing.
pub struct AnimationPlayer2DPlugin;

impl Plugin for AnimationPlayer2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_update_internal)
            .add_system(animation_player.after(animation_update_internal));
    }
}

#[derive(Reflect)]
struct PlayingAnimation2D {
    repeat: bool,
    speed: f32,
    elapsed: f32,
    animation_clip: Handle<AnimationClip2D>,
}

impl Default for PlayingAnimation2D {
    fn default() -> Self {
        Self {
            repeat: false,
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
    update_internal: bool,
}

impl AnimationPlayer2D {
    /// Start playing an animation, resetting state of the player
    pub fn start(&mut self, handle: Handle<AnimationClip2D>) -> &mut Self {
        self.animation = PlayingAnimation2D {
            animation_clip: handle,
            ..Default::default()
        };
        self.update_internal = true;

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

// TODO: Actual support the playback of sprite animations
/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn animation_player(
    time: Res<Time>,
    spritesheet_animationclips: Res<Assets<AnimationClip2D>>,
    mut query: Query<(&mut AnimationPlayer2D, &mut TextureAtlasSprite)>,
) {
    query.par_for_each_mut(32, |(player, sprite)| {
        run_animation_player(&time, &spritesheet_animationclips, player, sprite);
    });
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn run_animation_player(
    time: &Time,
    spritesheet_animationclips: &Assets<AnimationClip2D>,
    mut player: Mut<AnimationPlayer2D>,
    mut sprite: Mut<TextureAtlasSprite>,
) {
    let paused = player.paused;
    if paused && !player.is_changed() {
        // Allows manual update of elapsed when paused
        return;
    }

    apply_animation_player(
        time,
        spritesheet_animationclips,
        &mut player.animation,
        paused,
        &mut sprite.index,
    );
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn apply_animation_player(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    animation: &mut PlayingAnimation2D,
    paused: bool,
    sprite_index: &mut usize,
) {
    if let Some(animation_clip) = animation_clips.get(&animation.animation_clip) {
        // TODO: figure out something better
        if let Keyframes2D::Sprite(_) = animation_clip.keyframes {
            panic!("Your are using an AnimationClip2D with sprite keyframes, but you are using a TextureAtlas");
        }

        // Advance timer
        if !paused {
            animation.elapsed += time.delta_seconds() * animation.speed;
        }

        let mut elapsed = animation.elapsed;
        if animation.repeat {
            elapsed %= animation_clip.duration;
        }
        if elapsed < 0.0 {
            elapsed += animation_clip.duration;
        }

        let index = match animation_clip
            .keyframe_timestamps
            .binary_search_by(|probe| probe.partial_cmp(&elapsed).unwrap())
        {
            Ok(n) if n >= animation_clip.keyframe_timestamps.len() - 1 => return, // this clip is finished
            Ok(i) => i,
            Err(0) => return, // this clip isn't started yet
            Err(n) if n > animation_clip.keyframe_timestamps.len() => return, // this clip is finished TODO: Would this not also skip the last keyframe for 3D?
            Err(i) => i-1,
        };

        if let Keyframes2D::SpriteSheet(_, vec) = &animation_clip.keyframes {
            *sprite_index = vec[index]
        };
    }
}

/// Updates animation player internal state when chaning animation.
fn animation_update_internal(
    animation_clips: Res<Assets<AnimationClip2D>>,
    mut query: Query<
        (&mut AnimationPlayer2D, &mut Handle<TextureAtlas>),
        Changed<AnimationPlayer2D>,
    >,
) {
    for (mut player, mut texture_atlas_handle) in &mut query {
        if let Some(animation_clip) = animation_clips.get(&player.animation.animation_clip) {
            if player.update_internal {
                // Get TextureAtlas and update handle
                if let Keyframes2D::SpriteSheet(handle, _) = &animation_clip.keyframes {
                    *texture_atlas_handle = handle.clone_weak();
                }

                // Reset dirty flag
                player.update_internal = false;
            }
        }
    }
}
