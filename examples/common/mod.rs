mod camera_free;
pub use camera_free::*;

mod editor;
pub use editor::*;

use bevy::prelude::*;
use bevy_mod_skinned_aabb::SkinnedAabbPlugin;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, generate_mipmaps};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MipmapGeneratorPlugin, // generate mipmaps for better texture sampling
            SkinnedAabbPlugin, // aabb for skinned meshes
            // local
            EditorPlugin, // egui inspector
            CameraFreePlugin, // camera controls
        ))
        .add_systems(Update, generate_mipmaps::<StandardMaterial>); 
    }
}

