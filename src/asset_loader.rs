use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::Handle,
    sprite::TextureAtlas,
    utils::{BoxedFuture, HashMap, HashSet}, reflect::TypeUuid,
};
use serde::Deserialize;
use std::path::PathBuf;

/// Loader for spritesheet animation manifest files written in ron. Loads an SpriteSheetAnimationSet asset.
#[derive(Default)]
pub struct SpriteSheetAnimationLoader;

/// File extension for spritesheet animation manifest files written in ron.
const FILE_EXTENSIONS: &[&str] = &["trickfilm"];

impl AssetLoader for SpriteSheetAnimationLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let spritesheet_animationset_manifest = ron::de::from_bytes::<SpriteSheetAnimationSetManifest>(bytes)?;

            let mut spritesheet_animationset = SpriteSheetAnimationSet {
                name: spritesheet_animationset_manifest.name,
                ..Default::default()
            };
            let mut dependencies = HashSet::new();
            for (animation_name,spritesheet_animation_manifest) in spritesheet_animationset_manifest.animations {
                let spritesheet_animation_asset_path = AssetPath::new(PathBuf::from(&spritesheet_animation_manifest.path), None);
                dependencies.insert(spritesheet_animation_asset_path.clone());

                let texture_atlas_handle: Handle<TextureAtlas> = load_context.get_handle(spritesheet_animation_asset_path.clone());
                let spritesheet_animation = SpriteSheetAnimation {
                    texture_atlas_handle,
                    repeating: spritesheet_animation_manifest.repeating,
                    fps: spritesheet_animation_manifest.fps,
                    indices: spritesheet_animation_manifest.indices,
                };
                spritesheet_animationset.animations.insert(animation_name, spritesheet_animation);
            }

            let mut spritesheet_animation_asset = LoadedAsset::new(spritesheet_animationset);
            for dependency in dependencies {
                spritesheet_animation_asset.add_dependency(dependency);
            }

            load_context.set_default_asset(spritesheet_animation_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

#[derive(Debug, Deserialize)]
struct SpriteSheetAnimationSetManifest {
    #[serde(default)]
    name: Option<String>,
    animations: HashMap<String, SpriteSheetAnimationManifest>,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Deserialize)]
struct SpriteSheetAnimationManifest {
    path: String,
    repeating: bool,
    fps: usize,
    indices: Vec<usize>,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Default, TypeUuid)]
#[uuid = "ec942212-87dc-4ee4-8300-1e160a389c37"]
pub struct SpriteSheetAnimationSet {
    pub name: Option<String>,
    pub animations: HashMap<String, SpriteSheetAnimation>,
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
#[derive(Debug, Default)]
pub struct SpriteSheetAnimation {
    pub texture_atlas_handle: Handle<TextureAtlas>,
    pub repeating: bool,
    pub fps: usize,
    pub indices: Vec<usize>,
}

