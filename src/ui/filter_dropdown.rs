use bevy::{
    ecs::{children, component::Component, hierarchy::ChildOf, spawn::Spawn},
    prelude::*,
    ui::{AlignItems, Display, FlexDirection, GlobalZIndex, OverflowAxis, PositionType, Val},
    ui_widgets::observe,
    feathers::{
        controls::*,
        rounded_corners::RoundedCorners,
        theme::{ThemeBackgroundColor, ThemedText},
        tokens,
    },
};
use bevy_ui_text_input::TextInputBuffer;
use std::marker::PhantomData;
use strum::IntoEnumIterator;

use super::{
    scroll::{scroll, ScrollProps},
    text_input::{text_input, TextInputProps},
};

/// Marker for filter dropdown of type T
#[derive(Component)]
pub struct FilterDropdown<T: Component + Copy + Send + Sync + 'static> {
    pub target: Entity,
    pub is_open: bool,
    _marker: PhantomData<T>,
}

impl<T: Component + Copy + Send + Sync + 'static> FilterDropdown<T> {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            is_open: false,
            _marker: PhantomData,
        }
    }
}

/// Marker for the trigger button
#[derive(Component)]
pub struct FilterDropdownTrigger<T: Component + Copy + Send + Sync + 'static>(PhantomData<T>);

impl<T: Component + Copy + Send + Sync + 'static> Default for FilterDropdownTrigger<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Marker for trigger text
#[derive(Component)]
pub struct FilterDropdownTriggerText<T: Component + Copy + Send + Sync + 'static>(PhantomData<T>);

impl<T: Component + Copy + Send + Sync + 'static> Default for FilterDropdownTriggerText<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Marker for the options container
#[derive(Component)]
pub struct FilterDropdownOptions<T: Component + Copy + Send + Sync + 'static>(PhantomData<T>);

impl<T: Component + Copy + Send + Sync + 'static> Default for FilterDropdownOptions<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Marker for the filter text input
#[derive(Component)]
pub struct FilterDropdownInput<T: Component + Copy + Send + Sync + 'static>(PhantomData<T>);

impl<T: Component + Copy + Send + Sync + 'static> Default for FilterDropdownInput<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

/// Individual option storing the enum value
#[derive(Component)]
pub struct FilterDropdownOption<T: Component + Copy + Send + Sync + 'static> {
    pub value: T,
}

/// Creates a filterable dropdown for enum type T that updates component on target entity
///
/// # Example
/// ```ignore
/// parent.spawn(dropdown_filter::<SkinMesh>(human_entity, Some(current_skin_mesh)));
/// ```
pub fn dropdown_filter<T>(target: Entity, initial: Option<T>) -> impl Bundle
where
    T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static,
{
    let initial_text = initial
        .map(|v| v.to_string())
        .unwrap_or_else(|| "Select...".to_string());

    (
        Node {
            width: Val::Px(280.0),
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Relative,
            ..default()
        },
        FilterDropdown::<T>::new(target),
        children![
            // Trigger button
            (
                button(
                    ButtonProps::default(),
                    FilterDropdownTrigger::<T>::default(),
                    Spawn((
                        Text::new(initial_text),
                        ThemedText,
                        FilterDropdownTriggerText::<T>::default()
                    )),
                ),
                observe(on_filter_trigger_click::<T>),
            ),
            // Options container (hidden by default)
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                FilterDropdownOptions::<T>::default(),
                GlobalZIndex(1000),
                ThemeBackgroundColor(tokens::BUTTON_BG),
                children![
                    // Filter input
                    text_input(
                        TextInputProps {
                            width: Val::Percent(100.0),
                            height: Val::Px(32.0),
                            placeholder: "Filter...".to_string(),
                            corners: RoundedCorners::Top,
                            ..default()
                        },
                        FilterDropdownInput::<T>::default(),
                    ),
                    // Scrollable options
                    scroll(
                        ScrollProps {
                            width: Val::Percent(100.0),
                            height: Val::Px(300.0),
                            overflow: bevy::ui::Overflow {
                                x: OverflowAxis::Visible,
                                y: OverflowAxis::Scroll,
                            },
                            flex_direction: FlexDirection::Column,
                            corners: RoundedCorners::Bottom,
                            bg_token: tokens::BUTTON_BG,
                            align_items: AlignItems::Stretch,
                        },
                        (),
                        build_options::<T>(),
                    ),
                ],
            ),
        ],
    )
}

/// Build option buttons for all enum variants
fn build_options<T>() -> impl Bundle
where
    T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static,
{
    let options: Vec<_> = T::iter()
        .map(|value| {
            let label = value.to_string();
            (
                Name::new(label.clone()),
                button(ButtonProps::default(), (), Spawn((Text::new(label), ThemedText))),
                FilterDropdownOption { value },
                observe(on_filter_option_click::<T>),
            )
        })
        .collect();
    Children::spawn(SpawnIter(options.into_iter()))
}

fn on_filter_trigger_click<T>(
    trigger: On<Pointer<Click>>,
    trigger_query: Query<&ChildOf, With<FilterDropdownTrigger<T>>>,
    mut dropdown_query: Query<&mut FilterDropdown<T>>,
    child_of_query: Query<&ChildOf>,
) where
    T: Component + Copy + Send + Sync + 'static,
{
    let mut current = trigger.entity;

    // Direct check
    if let Ok(child_of) = trigger_query.get(current) {
        if let Ok(mut dropdown) = dropdown_query.get_mut(child_of.parent()) {
            dropdown.is_open = !dropdown.is_open;
            return;
        }
    }

    // Walk up hierarchy
    while let Ok(child_of) = child_of_query.get(current) {
        current = child_of.parent();
        if let Ok(trigger_child_of) = trigger_query.get(current) {
            if let Ok(mut dropdown) = dropdown_query.get_mut(trigger_child_of.parent()) {
                dropdown.is_open = !dropdown.is_open;
                return;
            }
        }
    }
}

fn on_filter_option_click<T>(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    option_query: Query<&FilterDropdownOption<T>>,
    mut dropdown_query: Query<(&mut FilterDropdown<T>, &Children)>,
    child_of_query: Query<&ChildOf>,
    trigger_query: Query<&Children, With<FilterDropdownTrigger<T>>>,
    mut text_query: Query<&mut Text, With<FilterDropdownTriggerText<T>>>,
) where
    T: Component + Copy + ToString + Send + Sync + 'static,
{
    // Walk up from clicked entity to find the option
    let mut current = trigger.entity;
    let option_value = loop {
        if let Ok(option) = option_query.get(current) {
            break option.value;
        }
        if let Ok(child_of) = child_of_query.get(current) {
            current = child_of.parent();
        } else {
            return; // Not an option click
        }
    };

    // Continue walking up to find the dropdown
    loop {
        if let Ok((mut dropdown, dropdown_children)) = dropdown_query.get_mut(current) {
            // Close dropdown
            dropdown.is_open = false;

            // Update trigger text
            for &child in dropdown_children {
                if let Ok(trigger_children) = trigger_query.get(child) {
                    for &text_child in trigger_children {
                        if let Ok(mut text) = text_query.get_mut(text_child) {
                            text.0 = option_value.to_string();
                        }
                    }
                }
            }

            // Set component on target entity
            commands.entity(dropdown.target).insert(option_value);
            return;
        }

        if let Ok(child_of) = child_of_query.get(current) {
            current = child_of.parent();
        } else {
            return;
        }
    }
}

/// System: update visibility based on is_open state
pub fn update_filter_dropdown_visibility<T>(
    changed: Query<(&FilterDropdown<T>, &Children), Changed<FilterDropdown<T>>>,
    trigger_query: Query<&FilterDropdownTrigger<T>>,
    mut style_query: Query<&mut Node, With<FilterDropdownOptions<T>>>,
) where
    T: Component + Copy + Send + Sync + 'static,
{
    for (dropdown, children) in changed.iter() {
        for &child in children {
            if trigger_query.get(child).is_ok() {
                continue;
            }
            if let Ok(mut node) = style_query.get_mut(child) {
                node.display = if dropdown.is_open {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}

/// System: filter options based on text input
pub fn filter_dropdown_options<T>(
    input_query: Query<(&TextInputBuffer, &ChildOf), (With<FilterDropdownInput<T>>, Changed<TextInputBuffer>)>,
    option_query: Query<(Entity, &FilterDropdownOption<T>)>,
    mut node_query: Query<&mut Node>,
    child_of_query: Query<&ChildOf>,
    dropdown_query: Query<Entity, With<FilterDropdown<T>>>,
) where
    T: Component + Copy + ToString + Send + Sync + 'static,
{
    // When text input changes, find which dropdown it belongs to and filter its options
    for (buffer, input_parent) in input_query.iter() {
        let filter_text = buffer.get_text().to_lowercase();

        // Walk up to find the dropdown entity
        let mut current = input_parent.parent();
        let dropdown_entity = loop {
            if dropdown_query.get(current).is_ok() {
                break current;
            }
            if let Ok(parent) = child_of_query.get(current) {
                current = parent.parent();
            } else {
                break current; // Give up, use current
            }
        };

        // Filter all options that belong to this dropdown (by walking up from each option)
        for (option_entity, option) in option_query.iter() {
            // Check if this option belongs to our dropdown
            let mut opt_current = option_entity;
            let belongs_to_dropdown = loop {
                if opt_current == dropdown_entity {
                    break true;
                }
                if let Ok(parent) = child_of_query.get(opt_current) {
                    opt_current = parent.parent();
                } else {
                    break false;
                }
            };

            if !belongs_to_dropdown {
                continue;
            }

            // Apply filter - use Display::None to hide, Display::Flex to show
            let visible = if filter_text.is_empty() {
                true
            } else {
                option.value.to_string().to_lowercase().contains(&filter_text)
            };

            if let Ok(mut node) = node_query.get_mut(option_entity) {
                node.display = if visible {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
    }
}
