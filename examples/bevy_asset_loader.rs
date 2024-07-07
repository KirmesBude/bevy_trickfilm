//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! and https://github.com/bevyengine/bevy/blob/v0.9.1/examples/animation/animated_fox.rs
//! Renders an animated sprite by loading all animation frames from multiple sprites
//! and changing the displayed image periodically.

#[path = "helpers/animation_controller.rs"]
mod animation_helper;

use animation_helper::keyboard_animation_control_helper;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_trickfilm::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .init_state::<MyStates>()
        .add_plugins(Animation2DPlugin)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_systems(
            Update,
            keyboard_animation_control.run_if(in_state(MyStates::Next)),
        )
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(texture_atlas_layout(tile_size_x = 24, tile_size_y = 24, columns = 8, rows = 1))]
    gabe_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "gabe-idle-run.png")]
    gabe_texture: Handle<Image>,
    #[asset(
        paths("gabe-idle-run.trickfilm#run", "gabe-idle-run.trickfilm#idle"),
        collection(typed)
    )]
    animations: Vec<Handle<AnimationClip2D>>,
}

fn setup(mut commands: Commands, my_assets: Res<MyAssets>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Prepare AnimationPlayer
    let mut animation_player = AnimationPlayer2D::default();
    animation_player
        .play(my_assets.animations[0].clone_weak())
        .repeat();

    // SpriteSheet entity
    commands
        .spawn(SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            texture: my_assets.gabe_texture.clone(),
            ..Default::default()
        })
        .insert(TextureAtlas {
            layout: my_assets.gabe_layout.clone(),
            ..Default::default()
        })
        .insert(animation_player);
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_player: Query<&mut AnimationPlayer2D>,
    my_assets: Res<MyAssets>,
    mut current_animation: Local<usize>,
    mut instructions_printed: Local<bool>,
) {
    if let Ok(mut player) = animation_player.get_single_mut() {
        keyboard_animation_control_helper(
            &keyboard_input,
            &mut player,
            &my_assets.animations,
            &mut current_animation,
            &mut instructions_printed,
        );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
