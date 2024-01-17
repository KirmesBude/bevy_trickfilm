use bevy::{animation::RepeatAnimation, prelude::*};
use bevy_trickfilm::prelude::*;

pub fn keyboard_animation_control(
    keyboard_input: &Input<KeyCode>,
    animation_player: &mut AnimationPlayer2D,
    animations: &[Handle<AnimationClip2D>],
    current_animation: &mut usize,
    instructions_printed: &mut bool,
) {
    if !*instructions_printed {
        println!("Animation controls:");
        println!("  - spacebar: play / pause");
        println!("  - arrow up / down: speed up / slow down animation playback");
        println!("  - arrow left / right: seek backward / forward");
        println!("  - digit 1 / 3 / 5: play the animation <digit> times");
        println!("  - L: loop the animation forever");
        println!("  - return: change animation");

        *instructions_printed = true;
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        if animation_player.is_paused() {
            animation_player.resume();
        } else {
            animation_player.pause();
        }
    }

    if keyboard_input.just_pressed(KeyCode::Up) {
        let speed = animation_player.speed();
        animation_player.set_speed(speed * 1.2);
    }

    if keyboard_input.just_pressed(KeyCode::Down) {
        let speed = animation_player.speed();
        animation_player.set_speed(speed * 0.8);
    }

    if keyboard_input.just_pressed(KeyCode::Left) {
        let elapsed = animation_player.seek_time();
        animation_player.seek_to(elapsed - 0.1);
    }

    if keyboard_input.just_pressed(KeyCode::Right) {
        let elapsed = animation_player.seek_time();
        animation_player.seek_to(elapsed + 0.1);
    }

    if keyboard_input.just_pressed(KeyCode::Return) {
        *current_animation = (*current_animation + 1) % animations.len();
        animation_player
            .play(animations[*current_animation].clone_weak())
            .repeat();
    }

    if keyboard_input.just_pressed(KeyCode::Key1) {
        animation_player.set_repeat(RepeatAnimation::Count(1));
        animation_player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::Key3) {
        animation_player.set_repeat(RepeatAnimation::Count(3));
        animation_player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::Key5) {
        animation_player.set_repeat(RepeatAnimation::Count(5));
        animation_player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::L) {
        animation_player.set_repeat(RepeatAnimation::Forever);
    }
}
