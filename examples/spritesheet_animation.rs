//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;
use bevy_trickfilm::{animation::SpriteSheetAnimationPlayer, SpriteSheetAnimationLoaderPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugin(SpriteSheetLoaderPlugin)
        .add_plugin(SpriteSheetAnimationLoaderPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let spritesheet_animationset_handle = asset_server.load("gabe-idle-run.trickfilm");
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        SpriteSheetAnimationPlayer::new(spritesheet_animationset_handle)
            .with_animation(String::from("run").into()),
    ));
}
