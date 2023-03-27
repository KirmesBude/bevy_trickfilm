use bevy::{
    prelude::{Assets, DetectChanges, Handle, Mut, Query, Res},
    sprite::{TextureAtlas, TextureAtlasSprite},
    time::Time,
};

use crate::asset::{AnimationClip2D, Keyframes2D};

use super::{AnimationPlayer2D, PlayingAnimation2D};

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
pub fn animation_player_spritesheet(
    time: Res<Time>,
    animation_clips: Res<Assets<AnimationClip2D>>,
    mut query: Query<(
        &mut AnimationPlayer2D,
        &mut TextureAtlasSprite,
        &mut Handle<TextureAtlas>,
    )>,
) {
    query
        .par_iter_mut()
        .for_each_mut(|(player, sprite, texture_atlas_handle)| {
            run_animation_player_spritesheet(
                &time,
                &animation_clips,
                player,
                sprite,
                texture_atlas_handle,
            );
        });
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn run_animation_player_spritesheet(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    mut player: Mut<AnimationPlayer2D>,
    mut sprite: Mut<TextureAtlasSprite>,
    texture_atlas_handle: Mut<Handle<TextureAtlas>>,
) {
    // Allow manual update of elapsed when paused
    let paused = player.paused;
    if paused && !player.is_changed() {
        return;
    }

    apply_animation_player_spritesheet(
        time,
        animation_clips,
        &mut player.animation,
        paused,
        &mut sprite.index,
        texture_atlas_handle,
    );
}

/// Updates animation player and forwards changes of the frame to the TextureAtlasSprite component.
fn apply_animation_player_spritesheet(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    animation: &mut PlayingAnimation2D,
    paused: bool,
    sprite_index: &mut usize,
    mut texture_atlas_handle: Mut<Handle<TextureAtlas>>,
) {
    if let Some(animation_clip) = animation_clips.get(&animation.animation_clip) {
        if let Keyframes2D::SpriteSheet(handle, vec) = &animation_clip.keyframes {
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
                Err(n) if n > animation_clip.keyframe_timestamps.len() => return, // this clip is finished
                Err(i) => i - 1,
            };

            *texture_atlas_handle = handle.clone_weak();
            *sprite_index = vec[index]
        };
    }
}
