use bevy::{
    prelude::{Assets, DetectChanges, Handle, Image, Mut, Query, Res},
    time::Time,
};

use crate::asset::{AnimationClip2D, Keyframes2D};

use super::{AnimationPlayer2D, PlayingAnimation2D};

/// System that will play all sprite, using any entity with an [`AnimationPlayer2D`]
/// and a [`Handle<AnimationClip2D>`] as an animation root.
pub fn animation_player_sprite(
    time: Res<Time>,
    animation_clips: Res<Assets<AnimationClip2D>>,
    mut query: Query<(&mut AnimationPlayer2D, &mut Handle<Image>)>,
) {
    query.par_iter_mut().for_each_mut(|(player, image_handle)| {
        run_animation_player_sprite(&time, &animation_clips, player, image_handle);
    });
}

fn run_animation_player_sprite(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    mut player: Mut<AnimationPlayer2D>,
    image_handle: Mut<Handle<Image>>,
) {
    // Allow manual update of elapsed when paused
    let paused = player.paused;
    if paused && !player.is_changed() {
        return;
    }

    apply_animation_player_sprite(
        time,
        animation_clips,
        &mut player.animation,
        paused,
        image_handle,
    );
}

fn apply_animation_player_sprite(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    animation: &mut PlayingAnimation2D,
    paused: bool,
    mut image_handle: Mut<Handle<Image>>,
) {
    if let Some(animation_clip) = animation_clips.get(&animation.animation_clip) {
        if let Keyframes2D::Sprite(image_handles) = animation_clip.keyframes() {
            // Advance timer
            if !paused {
                animation.elapsed += time.delta_seconds() * animation.speed;
            }

            let mut elapsed = animation.elapsed;
            if animation.repeat {
                elapsed %= animation_clip.duration();
            }
            if elapsed < 0.0 {
                elapsed += animation_clip.duration();
            }

            let index = match animation_clip
                .keyframe_timestamps()
                .binary_search_by(|probe| probe.partial_cmp(&elapsed).unwrap())
            {
                Ok(n) if n >= animation_clip.keyframe_timestamps().len() - 1 => return, // this clip is finished
                Ok(i) => i,
                Err(0) => return, // this clip isn't started yet
                Err(n) if n > animation_clip.keyframe_timestamps().len() => return, // this clip is finished
                Err(i) => i - 1,
            };

            *image_handle = image_handles[index].clone_weak();
        };
    }
}
