use bevy::{
    feathers::{
        controls::{SliderProps, slider},
        theme::ThemedText,
    },
    prelude::*,
    ui_widgets::{SliderValue, ValueChange, observe},
};

pub fn offset_slider<T: Component + Default + From<f32>>(
    human_entity: Entity,
    label: &'static str,
    value: f32,
    min: f32,
    max: f32,
) -> impl Bundle {
    (
        Name::new(format!("Slider{}", label)),
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        children![
            (
                Text::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                ThemedText
            ),
            (
                slider(SliderProps { value, min, max }, ()),
                observe(on_offset_change::<T>(human_entity)),
            ),
        ],
    )
}

fn on_offset_change<T: Component + Default + From<f32>>(
    human_entity: Entity,
) -> impl FnMut(On<ValueChange<f32>>, Commands) {
    move |trigger: On<ValueChange<f32>>, mut commands: Commands| {
        commands.entity(human_entity).insert(T::from(trigger.value));
        commands
            .entity(trigger.source)
            .insert(SliderValue(trigger.value));
    }
}
