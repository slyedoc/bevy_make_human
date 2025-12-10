pub mod text_input;
pub mod dropdown;
pub mod scroll;

use bevy::{picking::PickingSystems, prelude::*};

/// Plugin which registers the dropdown systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            PreUpdate,
            dropdown::update_dropdown_visibility.in_set(PickingSystems::Last),
        );
    }
}