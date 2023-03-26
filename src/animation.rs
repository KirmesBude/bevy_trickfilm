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

use crate::prelude::SpriteSheetAnimationClip;
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
pub struct SpriteSheetAnimationPlayerPlugin;

impl Plugin for SpriteSheetAnimationPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animation_update_internal)
            .add_system(animation_player.after(animation_update_internal));
    }
}

#[derive(Reflect)]
struct SpriteSheetPlayingAnimation {
    repeat: bool,
    speed: f32,
    elapsed: f32,
    animation_clip: Handle<SpriteSheetAnimationClip>,
}

impl Default for SpriteSheetPlayingAnimation {
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
pub struct SpriteSheetAnimationPlayer {
    paused: bool,
    animation: SpriteSheetPlayingAnimation,
    update_internal: bool,
}

impl SpriteSheetAnimationPlayer {
    /// Start playing an animation, resetting state of the player
    pub fn start(&mut self, handle: Handle<SpriteSheetAnimationClip>) -> &mut Self {
        self.animation = SpriteSheetPlayingAnimation {
            animation_clip: handle,
            ..Default::default()
        };
        self.update_internal = true;

        self
    }

    /// Start playing an animation, resetting state of the player, unless the requested animation is already playing.
    pub fn play(&mut self, handle: Handle<SpriteSheetAnimationClip>) -> &mut Self {
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

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn animation_player(
    time: Res<Time>,
    spritesheet_animationclips: Res<Assets<SpriteSheetAnimationClip>>,
    mut query: Query<(&mut SpriteSheetAnimationPlayer, &mut TextureAtlasSprite)>,
) {
    query.par_for_each_mut(32, |(player, sprite)| {
        run_animation_player(&time, &spritesheet_animationclips, player, sprite);
    });
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn run_animation_player(
    time: &Time,
    spritesheet_animationclips: &Assets<SpriteSheetAnimationClip>,
    mut player: Mut<SpriteSheetAnimationPlayer>,
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
    spritesheet_animationclips: &Assets<SpriteSheetAnimationClip>,
    animation: &mut SpriteSheetPlayingAnimation,
    paused: bool,
    sprite_index: &mut usize,
) {
    if let Some(animation_clip) = spritesheet_animationclips.get(&animation.animation_clip) {
        // Advance timer
        if !paused {
            animation.elapsed += time.delta_seconds() * animation.speed;
        }
        let animation_clip_duration =
            (animation_clip.indices.len() as f32) / (animation_clip.fps as f32);
        let mut elapsed = animation.elapsed;
        if animation.repeat {
            elapsed %= animation_clip_duration;
        }
        if elapsed < 0.0 {
            elapsed += animation_clip_duration;
        }

        let index = (elapsed * (animation_clip.fps as f32)).trunc() as usize;
        let index = index.min(animation_clip.indices.len() - 1); // TODO: Ensure that AnimationClips are never empty
        let index = animation_clip.indices[index];
        *sprite_index = index;
    }
}

/// Updates animation player internal state when chaning animation.
fn animation_update_internal(
    spritesheet_animationclips: Res<Assets<SpriteSheetAnimationClip>>,
    mut query: Query<
        (&mut SpriteSheetAnimationPlayer, &mut Handle<TextureAtlas>),
        Changed<SpriteSheetAnimationPlayer>,
    >,
) {
    for (mut player, mut texture_atlas_handle) in &mut query {
        if let Some(spritesheet_animationclip) =
            spritesheet_animationclips.get(&player.animation.animation_clip)
        {
            if player.update_internal {
                // Get TextureAtlas and update handle
                let animation_texture_atlas_handle =
                    spritesheet_animationclip.texture_atlas_handle.clone();
                *texture_atlas_handle = animation_texture_atlas_handle;

                // Reset dirty flag
                player.update_internal = false;
            }
        }
    }
}
