use std::cmp::max;

use bevy::{prelude::{Res, Assets, Query, Component, Handle, Changed, default}, time::{Time, Timer, TimerMode}, sprite::{TextureAtlasSprite, TextureAtlas}};

use crate::asset_loader::SpriteSheetAnimationSet;


pub(crate) fn animate_sprite(
    time: Res<Time>,
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<(
        &mut SpriteSheetAnimationPlayer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (mut player, mut sprite) in &mut query {
        // Get active animation
        if let Some(animation) = &player.animation {
            // Get AnimationSet
            let animationset_handle = player.animationset_handle.clone();
            let animationset = spritesheet_animationsets.get(&animationset_handle).unwrap();
            let spritesheet_animation = animationset.animations.get(animation).unwrap();

            // Advance timer
            player.timer.tick(time.delta());
            if player.timer.just_finished() {
                // Update player index
                let repeating = spritesheet_animation.repeating;
                let len = spritesheet_animation.indices.len();
                player.index = if repeating {
                    (player.index+1)%len
                } else {
                    // TODO: If we are at the end here we can disable the timer
                    max(player.index+1, len-1)
                };

                // Update texture atlas index
                sprite.index = spritesheet_animation.indices[player.index];
            }
        }
    }
}

pub(crate) fn animation_update_internal(
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<
        (&mut SpriteSheetAnimationPlayer,
            &mut Handle<TextureAtlas>),
        Changed<SpriteSheetAnimationPlayer>
    >,
) {
    for (mut player, mut texture_atlas_handle) in &mut query {
        if let Some(spritesheet_animationset) = spritesheet_animationsets.get(&player.animationset_handle) {
            if let Some(animation) = &player.animation {
                if player.update_internal {
                    // Get TextureAtlas and update handle
                    let animation = spritesheet_animationset.animations.get(animation).unwrap();
                    let animation_texture_atlas_handle = animation.texture_atlas_handle.clone();
                    *texture_atlas_handle = animation_texture_atlas_handle;

                    // Set up timer
                    let fps = animation.fps as f32;
                    let duration = 1.0/fps;
                    let mode = TimerMode::Repeating;
                    player.timer = Timer::from_seconds(duration, mode);

                    // Reset dirty flag
                    player.update_internal = false;
                }
            }
        }
    }
}

// TODO: Add some kind of state
#[derive(Debug, Component)]
pub struct SpriteSheetAnimationPlayer {
    animationset_handle: Handle<SpriteSheetAnimationSet>,
    animation: Option<String>,
    index: usize,
    timer: Timer,
    update_internal: bool,
}

impl Default for SpriteSheetAnimationPlayer {
    fn default() -> Self {
        Self { animationset_handle: Default::default(), animation: Default::default(), index: Default::default(), timer: Default::default(), update_internal: true }
    }
}

impl SpriteSheetAnimationPlayer {

    pub fn new(animationset_handle: Handle<SpriteSheetAnimationSet>) -> Self {
        Self {
            animationset_handle,
            ..default()
        }
    }

    pub fn with_animation(self, animation: String) -> Self {
        Self {
            animation: Some(animation),
            ..self
        }
    }

    pub fn play(&mut self, name: String) {
        self.animation = Some(name);
        self.index = usize::default();
        self.timer = Timer::default();
        self.update_internal = true;
    }
}