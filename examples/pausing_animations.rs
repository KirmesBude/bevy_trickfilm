//! This example demonstrates how to pause or resume animations based on the supplied `State`.

use bevy::{app::Animation, input::common_conditions::input_just_pressed, prelude::*};
use bevy_trickfilm::prelude::*;

/// This can also be done as a `SubState` or a `ComputedState`.
/// We use `app.configure_sets()` to toggle [`AnimationPlayer2DSystemSet`] to
/// only execute when we're in the `PauseState::Running` state.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .init_state::<PauseState>()
        .configure_sets(PostUpdate, Animation.run_if(in_state(PauseState::Running)))
        .add_plugins(Animation2DPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            toggle_animation_pause.run_if(input_just_pressed(KeyCode::Space)),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let atlas_texture = asset_server.load("gabe-idle-run.png");
    let texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(24, 24), 7, 1, None, None);
    let texture_atlas = TextureAtlas {
        layout: texture_atlas_layouts.add(texture_atlas_layout),
        ..Default::default()
    };

    // Camera
    commands.spawn(Camera2d);

    // Prepare AnimationPlayer
    let animation = asset_server.load("gabe-idle-run-animation.trickfilm.ron#run");

    let mut animation_player = AnimationPlayer2D::default();
    animation_player.play(animation.clone()).repeat();

    // SpriteSheet entity
    commands
        .spawn(Sprite {
            image: atlas_texture,
            texture_atlas: Some(texture_atlas),
            ..Default::default()
        })
        .insert(Transform::from_scale(Vec3::splat(6.0)))
        .insert(animation_player);

    println!("Pasuing controls:");
    println!("  - spacebar: play / pause");
}

fn toggle_animation_pause(
    current_pause_state: Res<State<PauseState>>,
    mut next_pause_state: ResMut<NextState<PauseState>>,
) {
    next_pause_state.set(match current_pause_state.get() {
        PauseState::Running => PauseState::Paused,
        PauseState::Paused => PauseState::Running,
    });
}
