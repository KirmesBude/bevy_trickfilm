use bevy::{
    prelude::{AppFunctionRegistry, Assets, DetectChanges, Mut, Query, Res},
    reflect::func::{ArgList, FunctionRegistry},
    sprite::TextureAtlas,
    time::Time,
};

use crate::asset::AnimationClip2D;

use super::{AnimationPlayer2D, PlayingAnimation2D};

/// System that will play all spritesheet animations, using any entity with an [`AnimationPlayer2D`]
/// and a [`Handle<AnimationClip2D>`] as an animation root.
pub(crate) fn animation_player_spritesheet(
    time: Res<Time>,
    animation_clips: Res<Assets<AnimationClip2D>>,
    mut query: Query<(&mut AnimationPlayer2D, &mut TextureAtlas)>,
    function_registry: Res<AppFunctionRegistry>,
) {
    query.par_iter_mut().for_each(|(player, sprite)| {
        run_animation_player_spritesheet(
            &time,
            &animation_clips,
            player,
            sprite,
            &function_registry.read(),
        );
    });
}

fn run_animation_player_spritesheet(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    mut player: Mut<AnimationPlayer2D>,
    mut texture_atlas: Mut<TextureAtlas>,
    function_registry: &FunctionRegistry,
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
        &mut texture_atlas.index,
        function_registry,
    );
}

fn apply_animation_player_spritesheet(
    time: &Time,
    animation_clips: &Assets<AnimationClip2D>,
    animation: &mut PlayingAnimation2D,
    paused: bool,
    texture_atlas_index: &mut usize,
    function_registry: &FunctionRegistry,
) {
    if let Some(animation_clip) = animation_clips.get(&animation.animation_clip) {
        // We don't return early because seek_to() may have been called on the animation player.
        animation.update(
            if paused { 0.0 } else { time.delta_seconds() },
            animation_clip.duration(),
        );

        let index = match animation_clip
            .keyframe_timestamps()
            .binary_search_by(|probe| {
                probe
                    .partial_cmp(&animation.seek_time)
                    .expect("Keyframe timestamps contain elements, that are not comparable.")
            }) {
            Ok(n) if n >= animation_clip.keyframe_timestamps().len() - 1 => return,
            Ok(i) => i,
            Err(0) => return, // this clip isn't started yet
            Err(n) if n > animation_clip.keyframe_timestamps().len() => return,
            Err(i) => i - 1,
        };

        // User callbacks
        for callback in animation_clip.callbacks() {
            let args = ArgList::new();
            function_registry.get(callback).unwrap().call(args).unwrap(); // TODO: Proper error handling
        }

        let keyframes = animation_clip.keyframes();
        *texture_atlas_index = keyframes[index]
    }
}
