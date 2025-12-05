mod camera_free;
pub use camera_free::*;

mod editor;
pub use editor::*;

use bevy::prelude::*;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EditorPlugin,
            CameraFreePlugin,
        ));
    }
}

