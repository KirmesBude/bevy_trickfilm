//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_helper;

use bevy::prelude::*;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(Animation2DPlugin) // This initializes everything for Time<()> aka Virtual
        .add_plugins(AnimationPlayer2DPlugin::<Real>::new())
        .add_systems(Startup, setup)
        .add_systems(Update, keyboard_animation_control)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    println!("Animation controls:");
    println!("  - spacebar: pause / unpause virtual time");

    // Load all animations
    let animations = vec![
        asset_server.load("gabe-idle-run-animation.trickfilm.ron#run"),
        asset_server.load("gabe-idle-run-animation.trickfilm.ron#idle"),
    ];

    let atlas_texture = asset_server.load("gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None);
    let texture_atlas = TextureAtlas {
        layout: texture_atlas_layouts.add(texture_atlas_layout),
        ..Default::default()
    };

    // Camera
    commands.spawn(Camera2d);

    // Prepare AnimationPlayer
    let mut animation_player = AnimationPlayer2D::<Real>::new();
    animation_player.play(animations[0].clone()).repeat();

    let mut animation_player_virtual = AnimationPlayer2D::default(); // Default is Virtual
    animation_player_virtual
        .play(animations[0].clone())
        .repeat();

    // SpriteSheet entity
    commands
        .spawn(Sprite {
            image: atlas_texture.clone(),
            texture_atlas: Some(texture_atlas.clone()),
            ..Default::default()
        })
        .insert(
            Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-200.0, 0.0, 0.0)),
        )
        .insert(animation_player);

    commands
        .spawn(Sprite {
            image: atlas_texture.clone(),
            texture_atlas: Some(texture_atlas.clone()),
            ..Default::default()
        })
        .insert(
            Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(200.0, 0.0, 0.0)),
        )
        .insert(animation_player_virtual);

    // Set virtual speed to 2x
    virtual_time.set_relative_speed(2.);
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}
