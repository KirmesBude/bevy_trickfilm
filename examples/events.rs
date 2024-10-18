//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_helper;

use animation_helper::keyboard_animation_control_helper;
use bevy::{prelude::*, utils::HashMap};
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(Animation2DPlugin)
        .add_event::<SampleEvent>()
        .add_animation_event::<SampleEvent>()
        .register_type::<SampleEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (keyboard_animation_control, update_frame_text, print_event),
        )
        .run();
}

#[derive(Debug, Clone, Event, Reflect)]
struct SampleEvent {
    msg: String,
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip2D>>);

#[derive(Component)]
struct FrameText;

fn setup(
    mut commands: Commands,
    mut clips: ResMut<Assets<AnimationClip2D>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        FrameText,
        TextBundle::from_section(
            "current frame: 0",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ),
    ));

    let start = SampleEvent {
        msg: String::from("start"),
    };
    let middle = SampleEvent {
        msg: String::from("middle"),
    };
    let end = SampleEvent {
        msg: String::from("end"),
    };

    // animation events
    let mut events = HashMap::new();
    events.insert(1, vec![start.as_reflect().clone_value()]);
    events.insert(3, vec![middle.as_reflect().clone_value()]);
    events.insert(6, vec![end.as_reflect().clone_value()]);

    // Load all animations
    let animations = vec![
        clips.add(
            AnimationClip2D::new(
                vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5],
                vec![1, 2, 3, 4, 5, 6],
                0.6,
                events,
            )
            .unwrap(),
        ),
        clips.add(AnimationClip2D::new(vec![0.0], vec![0], 0.1, HashMap::new()).unwrap()),
    ];

    let atlas_texture = asset_server.load("gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None);
    let texture_atlas = TextureAtlas {
        layout: texture_atlas_layouts.add(texture_atlas_layout),
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
        .spawn(SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture: atlas_texture,
            ..Default::default()
        })
        .insert(texture_atlas)
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

    text.sections[0].value = format!("current frame: {}", animation_player.frame());
}

fn print_event(mut event_reader: EventReader<SampleEvent>) {
    for event in event_reader.read() {
        println!("{}", event.msg);
    }
}
