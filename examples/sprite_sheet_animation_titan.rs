//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

use bevy::{animation::RepeatAnimation, prelude::*};
use bevy_titan::SpriteSheetLoaderPlugin;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_plugins(Animation2DPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (setup_scene_once_loaded, keyboard_animation_control),
        )
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip2D>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Insert a resource with the current animation information
    commands.insert_resource(Animations(vec![
        asset_server.load("gabe-idle-run.trickfilm#run"),
        asset_server.load("gabe-idle-run.trickfilm#idle"),
    ]));

    let atlas_texture =
        asset_server.load("spritesheet_animation_titan/gabe-idle-run.titan#texture");
    let texture_atlas = TextureAtlas {
        layout: asset_server.load("spritesheet_animation_titan/gabe-idle-run.titan#layout"),
        ..Default::default()
    };
    // Camera
    commands.spawn(Camera2dBundle::default());

    // SpriteSheet entity
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture: atlas_texture,
            atlas: texture_atlas,
            ..default()
        })
        .insert(AnimationPlayer2D::default());

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - digit 1 / 3 / 5: play the animation <digit> times");
    println!("  - L: loop the animation forever");
    println!("  - return: change animation");
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut player: Query<&mut AnimationPlayer2D>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok(mut player) = player.get_single_mut() {
            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
        }
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer2D>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    if let Ok(mut player) = animation_player.get_single_mut() {
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
            let animations = &animations.0;
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
}
