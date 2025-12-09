pub mod text_input;
pub mod dropdown;
pub mod filter_dropdown;
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

/// Register filter dropdown systems for a specific enum type T
/// Call this in your app setup for each enum type you want to use
pub fn register_filter_dropdown<T>(app: &mut App)
where
    T: Component + Copy + ToString + strum::IntoEnumIterator + Send + Sync + 'static,
{
    app.add_systems(
        PreUpdate,
        (
            filter_dropdown::update_filter_dropdown_visibility::<T>,
            filter_dropdown::filter_dropdown_options::<T>,
        )
            .in_set(PickingSystems::Last),
    );
}
