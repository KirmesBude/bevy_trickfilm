//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_state::<MyStates>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(Animation2DPlugin)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading).continue_to_state(MyStates::Next),
        )
        .add_collection_to_loading_state::<_, MyAssets>(MyStates::AssetLoading)
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_systems(
            Update,
            keyboard_animation_control.run_if(in_state(MyStates::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 24., tile_size_y = 24., columns = 7, rows = 1))]
    #[asset(path = "gabe-idle-run.png")]
    gabe: Handle<TextureAtlas>,
    #[asset(path = "gabe-idle-run.trickfilm")]
    animations: Handle<AnimationClip2DSet>,
}

fn setup(
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    animation_clip_2d_sets: Res<Assets<AnimationClip2DSet>>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let animations = animation_clip_2d_sets
        .get(my_assets.animations.clone_weak())
        .unwrap();

    // Prepare AnimationPlayer
    let mut animation_player = AnimationPlayer2D::default();
    animation_player
        .play(animations.animations["run"].clone_weak())
        .repeat();

    // SpriteSheet entity
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture_atlas: my_assets.gabe.clone(),
            ..default()
        })
        .insert(animation_player);

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - return: change animation");
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer2D>,
    my_assets: Res<MyAssets>,
    mut current_animation: Local<usize>,
    animation_clip_2d_sets: Res<Assets<AnimationClip2DSet>>,
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
            let animations = animation_clip_2d_sets
                .get(my_assets.animations.clone_weak())
                .unwrap();

            *current_animation = (*current_animation + 1) % 2;
            let animation = match *current_animation {
                0 => animations.animations["idle"].clone_weak(),
                1 => animations.animations["run"].clone_weak(),
                _ => !unreachable!(),
            };
            player.play(animation).repeat();
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
