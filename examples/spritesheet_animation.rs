//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugin(SpriteSheetLoaderPlugin)
        .add_plugin(SpriteSheetAnimationPlugin)
        .add_startup_system(setup)
        .add_system(setup_scene_once_loaded)
        .add_system(keyboard_animation_control)
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<SpriteSheetAnimationClip>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Animations(vec![
        asset_server.load("gabe-idle-run.trickfilm#run")
    ]));

    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        })
        .insert(SpriteSheetAnimationPlayer::default());
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut player: Query<&mut SpriteSheetAnimationPlayer>,
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
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_player: Query<&mut SpriteSheetAnimationPlayer>,
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

        if keyboard_input.just_pressed(KeyCode::Up) {
            let speed = player.speed();
            player.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::Down) {
            let speed = player.speed();
            player.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::Left) {
            let elapsed = player.elapsed();
            player.set_elapsed(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            let elapsed = player.elapsed();
            player.set_elapsed(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            player
                .play(animations.0[*current_animation].clone_weak())
                .repeat();
        }
    }
}
