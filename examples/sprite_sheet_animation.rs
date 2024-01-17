//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_controller;

use animation_controller::keyboard_animation_control;
use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(Animation2DPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, my_keyboard_animation_control)
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip2D>>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    let animations = Animations(vec![
        asset_server.load("gabe-idle-run.trickfilm#run"),
        asset_server.load("gabe-idle-run.trickfilm#idle"),
    ]);

    // Prepare AnimationPlayer
    let mut animation_player = AnimationPlayer2D::default();
    animation_player.play(animations.0[0].clone_weak()).repeat();

    // Insert a resource with the current animation information
    commands.insert_resource(animations);

    let texture_handle = asset_server.load("gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    // SpriteSheet entity
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture_atlas: texture_atlas_handle,
            ..default()
        })
        .insert(animation_player);
}

fn my_keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer2D>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
    mut instructions_printed: Local<bool>,
) {
    if let Ok(mut animation_player) = animation_players.get_single_mut() {
        keyboard_animation_control(
            &keyboard_input,
            &mut animation_player,
            &animations.0,
            &mut current_animation,
            &mut instructions_printed,
        );
    }
}
