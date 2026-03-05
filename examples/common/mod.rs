mod camera_free;
pub use camera_free::*;

mod editor;
pub use editor::*;

use bevy::prelude::*;
use bevy_mod_mipmap_generator::{MipmapGeneratorPlugin, generate_mipmaps};
use bevy_mod_skinned_aabb::SkinnedAabbPlugin;

/// The default [`LogPlugin`] [`EnvFilter`].
pub const FILTER: &str = concat!(
    "wgpu=error,",
    "naga=warn,",
    "symphonia_bundle_mp3::demuxer=warn,",
    "symphonia_format_caf::demuxer=warn,",
    "symphonia_format_isompf4::demuxer=warn,",
    "symphonia_format_mkv::demuxer=warn,",
    "symphonia_format_ogg::demuxer=warn,",
    "symphonia_format_riff::demuxer=warn,",
    "symphonia_format_wav::demuxer=warn,",
    "calloop::loop_logic=error,",
    "wgpu_hal::vulkan=off",
);

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MipmapGeneratorPlugin, // generate mipmaps for better texture sampling
            SkinnedAabbPlugin,     // aabb for skinned meshes
            // local
            EditorPlugin,     // egui inspector
            CameraFreePlugin, // camera controls
        ))
        .add_systems(Update, generate_mipmaps::<StandardMaterial>);
    }
}
