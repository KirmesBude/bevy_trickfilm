//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_helper;

use animation_helper::keyboard_animation_control_helper;
use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_plugins(Animation2DPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_animation_control)
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip2D>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load all animations
    let animations = vec![
        asset_server.load("gabe-idle-run.trickfilm#run"),
        asset_server.load("gabe-idle-run.trickfilm#idle"),
    ];

    let atlas_texture =
        asset_server.load("spritesheet_animation_titan/gabe-idle-run.titan#texture");
    let texture_atlas = TextureAtlas {
        layout: asset_server.load("spritesheet_animation_titan/gabe-idle-run.titan#layout"),
        ..Default::default()
    };

    // Camera
    commands.spawn(Camera2dBundle::default());

    // Prepare AnimationPlayer
    let mut animation_player = AnimationPlayer2D::default();
    animation_player.play(animations[0].clone_weak()).repeat();

    // Insert a resource with the current animation information
    commands.insert_resource(Animations(animations));

    // SpriteSheet entity
    commands
        .spawn(SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture: atlas_texture,
            atlas: texture_atlas,
            ..default()
        })
        .insert(animation_player);
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer2D>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
    mut instructions_printed: Local<bool>,
) {
    if let Ok(mut player) = animation_player.get_single_mut() {
        keyboard_animation_control_helper(
            &keyboard_input,
            &mut player,
            &animations.0,
            &mut current_animation,
            &mut instructions_printed,
        );
    }
}
