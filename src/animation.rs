//! This crate allows you to directly load a TextureAtlas from a manifest file.
//!
//! `bevy_titan` introduces a [`SpriteSheetManifest`](crate::SpriteSheetManifest) and the corresponding [`SpriteSheetLoader`](crate::SpriteSheetLoader).
//! Assets with the 'titan' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`TextureAtlas`](::bevy::sprite::TextureAtlas) [`Handle`](::bevy::asset::Handle).
//!
//! ### `spritesheet.titan`
//! ```rust,ignore
//! SpriteSheetManifest ( /* The explicit type name can be omitted */
//!     path: String, /* path to spritesheet image asset */
//!     tile_size: (
//!         w: f32,
//!         h: f32,
//!     ),
//!     columns: usize,
//!     rows: usize,
//!    // These can be optionally defined
//!    /*
//!    padding: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    offset: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    */
//! )
//! ```
//!
//! ```edition2021
//! # use bevy_titan::SpriteSheetLoaderPlugin;
//! # use bevy::prelude::*;
//! #
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(SpriteSheetLoaderPlugin)
//!         .add_system(load_spritesheet)
//!         .run();
//! }
//!
//! fn load_spritesheet(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let texture_atlas_handle = asset_server.load("spritesheet.titan");
//!     commands.spawn(Camera2dBundle::default());
//!     commands.spawn(
//!         SpriteSheetBundle {
//!              texture_atlas: texture_atlas_handle,
//!              transform: Transform::from_scale(Vec3::splat(6.0)),
//!              ..default()
//!         }
//!     );
//! }
//!
//! ```

use crate::asset_loader::SpriteSheetAnimationSet;
use bevy::{
    prelude::{default, Assets, Changed, Component, Handle, Query, Res},
    sprite::{TextureAtlas, TextureAtlasSprite},
    time::{Time, Timer, TimerMode},
};

pub(crate) fn animate_sprite(
    time: Res<Time>,
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<(&mut SpriteSheetAnimationPlayer, &mut TextureAtlasSprite)>,
) {
    for (mut player, mut sprite) in &mut query {
        if let SpriteSheetAnimationPlayerState::Stopped = player.state() {
            continue;
        }

        // Get active animation
        if let Some(animation) = player.animation() {
            // Get AnimationSet
            let animationset_handle = player.animationset_handle.clone();
            let animationset = spritesheet_animationsets.get(&animationset_handle).unwrap();
            let spritesheet_animation = animationset.animations.get(animation).unwrap();

            // Advance timer
            player.timer.tick(time.delta());
            if player.timer.just_finished() {
                // Update player index
                let repeating = spritesheet_animation.repeating;
                let len = spritesheet_animation.indices.len();
                player.index = if repeating {
                    (player.index + 1) % len
                } else {
                    // TODO: If we are at the end here we can disable the timer
                    (player.index + 1).max(len - 1)
                };

                // Update texture atlas index
                sprite.index = spritesheet_animation.indices[player.index];
            }
        }
    }
}

pub(crate) fn animation_update_internal(
    spritesheet_animationsets: Res<Assets<SpriteSheetAnimationSet>>,
    mut query: Query<
        (&mut SpriteSheetAnimationPlayer, &mut Handle<TextureAtlas>),
        Changed<SpriteSheetAnimationPlayer>,
    >,
) {
    for (mut player, mut texture_atlas_handle) in &mut query {
        if let Some(spritesheet_animationset) =
            spritesheet_animationsets.get(&player.animationset_handle)
        {
            if let Some(animation) = player.animation() {
                if player.update_internal {
                    // Get TextureAtlas and update handle
                    let animation = spritesheet_animationset.animations.get(animation).unwrap();
                    let animation_texture_atlas_handle = animation.texture_atlas_handle.clone();
                    *texture_atlas_handle = animation_texture_atlas_handle;

                    // Set up timer
                    let fps = animation.fps as f32;
                    let duration = 1.0 / fps;
                    let mode = TimerMode::Repeating;
                    player.timer = Timer::from_seconds(duration, mode);

                    // Reset dirty flag
                    player.update_internal = false;
                }
            }
        }
    }
}

#[derive(Debug, Component)]
pub struct SpriteSheetAnimationPlayer {
    animationset_handle: Handle<SpriteSheetAnimationSet>,
    state: SpriteSheetAnimationPlayerState,
    index: usize,
    timer: Timer,
    update_internal: bool,
}

impl Default for SpriteSheetAnimationPlayer {
    fn default() -> Self {
        Self {
            animationset_handle: Default::default(),
            state: Default::default(),
            index: Default::default(),
            timer: Default::default(),
            update_internal: true,
        }
    }
}

impl SpriteSheetAnimationPlayer {
    pub fn new(animationset_handle: Handle<SpriteSheetAnimationSet>) -> Self {
        Self {
            animationset_handle,
            ..default()
        }
    }

    pub fn with_animation(self, animation: String) -> Self {
        Self {
            state: SpriteSheetAnimationPlayerState::Playing(animation),
            ..self
        }
    }

    pub fn play(&mut self, name: String) {
        self.state = SpriteSheetAnimationPlayerState::Playing(name);
        self.index = usize::default();
        self.timer = Timer::default();
        self.update_internal = true;
    }

    pub fn animation(&self) -> Option<&str> {
        match &self.state {
            SpriteSheetAnimationPlayerState::Playing(animation) => Some(animation),
            SpriteSheetAnimationPlayerState::Stopped => None,
        }
    }

    pub fn state(&self) -> &SpriteSheetAnimationPlayerState {
        &self.state
    }
}

#[derive(Debug, Default)]
pub enum SpriteSheetAnimationPlayerState {
    Playing(String),
    #[default]
    Stopped,
}
