use bevy::{animation::RepeatAnimation, prelude::*};
use bevy_trickfilm::{animation::AnimationPlayer2D, asset::AnimationClip2D};

pub fn keyboard_animation_control_helper(
    keyboard_input: &ButtonInput<KeyCode>,
    player: &mut AnimationPlayer2D,
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
        if player.is_paused() {
            player.resume();
        } else {
            player.pause();
        }
    }

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        let speed = player.speed();
        player.set_speed(speed * 1.2);
    }

    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        let speed = player.speed();
        player.set_speed(speed * 0.8);
    }

    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        let elapsed = player.seek_time();
        player.seek_to(elapsed - 0.1);
    }

    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        let elapsed = player.seek_time();
        player.seek_to(elapsed + 0.1);
    }

    if keyboard_input.just_pressed(KeyCode::Enter) {
        *current_animation = (*current_animation + 1) % animations.len();
        player
            .play(animations[*current_animation].clone_weak())
            .repeat();
    }

    if keyboard_input.just_pressed(KeyCode::Digit1) {
        player.set_repeat(RepeatAnimation::Count(1));
        player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::Digit3) {
        player.set_repeat(RepeatAnimation::Count(3));
        player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::Digit5) {
        player.set_repeat(RepeatAnimation::Count(5));
        player.replay();
    }

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        player.set_repeat(RepeatAnimation::Forever);
    }
}
