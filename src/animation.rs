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

use crate::asset_loader::SpriteSheetAnimationSet;
use bevy::{
    prelude::{
        default, App, Assets, Changed, Component, Handle, IntoSystemDescriptor, Plugin, Query, Res,
    },
    sprite::{TextureAtlas, TextureAtlasSprite},
    time::{Time, Timer, TimerMode},
};

/// Adds support for spritesheet animation playing.
pub struct SpriteSheetAnimationPlayerPlugin;

impl Plugin for SpriteSheetAnimationPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_update_internal)
            .add_system(animate_sprite.after(animation_update_internal));
    }
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn animate_sprite(
    time: Res<Time>,
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<(&mut SpriteSheetAnimationPlayer, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut sprite) in &mut query {
        match player.state {
            SpriteSheetAnimationPlayerState::Stopped
            | SpriteSheetAnimationPlayerState::Paused(_) => continue,
            _ => {}
        }

        // Get active animation
        if let Some(animation) = player.animation() {
            // Get AnimationSet
            let animationset_handle = player.animationset_handle.clone();
            let animationset = spritesheet_animationsets.get(&animationset_handle).unwrap();
            let spritesheet_animation = animationset.animations.get(animation).unwrap();

            // Advance timer
            let speed = player.speed;
            player.timer.tick(time.delta().mul_f32(speed));
            if player.timer.just_finished() {
                // Update player index
                let repeating = spritesheet_animation.repeating;
                let len = spritesheet_animation.indices.len();
                let new_index = if repeating {
                    (player.index + 1) % len
                } else {
                    (player.index + 1).max(len - 1)
                };

                if player.index != new_index {
                    player.index = new_index;
                    // Update texture atlas index
                    sprite.index = spritesheet_animation.indices[player.index];
                } else {
                    player.stop();
                }
            }
        }
    }
}

/// Updates animation player internal state when chaning animation.
fn animation_update_internal(
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<
        (&mut SpriteSheetAnimationPlayer, &mut Handle<TextureAtlas>),
        Changed<SpriteSheetAnimationPlayer>,
    >,
) {
    for (mut player, mut texture_atlas_handle) in &mut query {
        if let Some(spritesheet_animationset) =
            spritesheet_animationsets.get(&player.animationset_handle)
        {
            if let Some(animation) = player.animation() {
                if player.update_internal {
                    // Get TextureAtlas and update handle
                    let animation = spritesheet_animationset.animations.get(animation).unwrap();
                    let animation_texture_atlas_handle = animation.texture_atlas_handle.clone();
                    *texture_atlas_handle = animation_texture_atlas_handle;

                    // Set up timer
                    let fps = animation.fps as f32;
                    let duration = 1.0 / fps;
                    let mode = TimerMode::Repeating;
                    player.timer = Timer::from_seconds(duration, mode);

                    // Reset dirty flag
                    player.update_internal = false;
                }
            }
        }
    }
}

/* TODO: Introduce Resource to handle stopping any animation updates/ticking + a way to override it per component */
/* TODO: Return Error from play animation */

/// Component to handle playing animations.
#[derive(Debug, Component)]
pub struct SpriteSheetAnimationPlayer {
    animationset_handle: Handle<SpriteSheetAnimationSet>,
    state: SpriteSheetAnimationPlayerState,
    index: usize,
    speed: f32,
    timer: Timer,
    update_internal: bool,
}

impl Default for SpriteSheetAnimationPlayer {
    fn default() -> Self {
        Self {
            animationset_handle: Default::default(),
            state: Default::default(),
            index: Default::default(),
            timer: Default::default(),
            speed: 1.0,
            update_internal: true,
        }
    }
}

impl SpriteSheetAnimationPlayer {
    /// Creates a new SpriteSheetAnimationPlayer from a SpriteSheetAnimationSet asset.
    pub fn new(animationset_handle: Handle<SpriteSheetAnimationSet>) -> Self {
        Self {
            animationset_handle,
            ..default()
        }
    }

    /// Creates a new SpriteSheetAnimationPlayer from a SpriteSheetAnimationSet asset.
    /// Will immediately start playing the given animation.
    pub fn with_animation(self, animation: String) -> Self {
        Self {
            state: SpriteSheetAnimationPlayerState::Playing(animation),
            ..self
        }
    }

    /// Plays the given animation.
    pub fn play(&mut self, name: String) {
        self.state = SpriteSheetAnimationPlayerState::Playing(name);
        self.index = usize::default();
        self.timer = Timer::default();
        self.update_internal = true;
    }

    /// Stops the currently playing animation.
    pub fn stop(&mut self) {
        self.state = SpriteSheetAnimationPlayerState::Stopped;
    }

    /// If there is currently an animation playing or paused, returns the name of the animation.
    pub fn animation(&self) -> Option<&str> {
        match &self.state {
            SpriteSheetAnimationPlayerState::Playing(animation)
            | SpriteSheetAnimationPlayerState::Paused(animation) => Some(animation),
            SpriteSheetAnimationPlayerState::Stopped => None,
        }
    }

    /// Returns the current animation state of the animation player.
    pub fn state(&self) -> &SpriteSheetAnimationPlayerState {
        &self.state
    }

    /// Returns the current animation speed of the animation player.
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Change the current animation speed of the animation player.
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Pauses the current animation. If there is no animation playing, does nothing.
    pub fn pause(&mut self) {
        if let SpriteSheetAnimationPlayerState::Playing(animation) = &self.state {
            self.state = SpriteSheetAnimationPlayerState::Paused(animation.to_owned())
        }
    }

    /// Pauses the current animation. If there is no animation playing, does nothing.
    pub fn resume(&mut self) {
        if let SpriteSheetAnimationPlayerState::Paused(animation) = &self.state {
            self.state = SpriteSheetAnimationPlayerState::Playing(animation.to_owned())
        }
    }
}

/// Animation state of the animation player.
#[derive(Debug, Default)]
pub enum SpriteSheetAnimationPlayerState {
    /// The animation with the given name is playing.
    Playing(String),
    #[default]
    /// No animation is currently playing.
    Stopped,
    /// The animation with the given name is paused.
    Paused(String),
}
