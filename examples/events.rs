//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_helper;

use animation_helper::keyboard_animation_control_helper;
use bevy::prelude::*;
use bevy_trickfilm::{animation::event::EventTarget, prelude::*};
use bevy_trickfilm_derive::AnimationEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(Animation2DPlugin)
        // add_animation_event will add the event to the app, register the type and setup trickfilm internal resources and systems
        .add_animation_event::<SampleEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_animation_control, update_frame_text, print_event),
        )
        .run();
}

// This Event needs to implement AnimationEvent
#[derive(Debug, Clone, Event, Reflect, AnimationEvent)]
struct SampleEvent {
    #[reflect(skip_serializing)]
    // This is necessary, because EventTarget is not given via the trickfilm file, but at runtime via the AnimationEvent trait
    #[animationevent(target)]
    target: EventTarget,
    msg: String,
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip2D>>);

#[derive(Component)]
struct FrameText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((FrameText, Text::new("current frame: 0")));

    let animations = vec![
        asset_server.load("gabe-idle-run-animation-events.trickfilm.ron#run"),
        asset_server.load("gabe-idle-run-animation-events.trickfilm.ron#idle"),
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
    let mut animation_player = AnimationPlayer2D::default();
    animation_player.play(animations[0].clone_weak()).repeat();

    // Insert a resource with the current animation information
    commands.insert_resource(Animations(animations));

    // SpriteSheet entity
    commands
        .spawn(Sprite {
            image: atlas_texture,
            texture_atlas: Some(texture_atlas),
            ..Default::default()
        })
        .insert(Transform::from_scale(Vec3::splat(6.0)))
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

fn update_frame_text(
    mut q_frame_text: Query<&mut Text, With<FrameText>>,
    q_animation_player: Query<&AnimationPlayer2D>,
) {
    let Ok(mut text) = q_frame_text.get_single_mut() else {
        return;
    };
    let Ok(animation_player) = q_animation_player.get_single() else {
        return;
    };

    text.0 = format!("current frame: {}", animation_player.frame());
}

// You can easily react on your custom event just like a normal bevy event
fn print_event(mut event_reader: EventReader<SampleEvent>) {
    for event in event_reader.read() {
        println!("{:?}", event);
    }
}
