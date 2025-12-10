use bevy::{
    ecs::{
        bundle::Bundle,
        children,
        component::Component,
        hierarchy::{ChildOf, Children},
        observer::On,
        query::{Changed, With},
        reflect::ReflectComponent,
        spawn::{Spawn, SpawnRelated, SpawnableList},
        system::{Commands, Query},
    },
    reflect::{prelude::ReflectDefault, Reflect},
    ui::{
        AlignItems, Display, FlexDirection, GlobalZIndex, Node, OverflowAxis,
        PositionType, Val,
    },
    ui_widgets::{observe, Activate, ValueChange},
    feathers::{
        controls::*,
        rounded_corners::RoundedCorners,
        theme::{ThemeBackgroundColor, ThemedText},
        tokens,
    },
    prelude::*,
};

use super::scroll::{scroll, ScrollProps};

/// Component marking the dropdown container
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug, Clone)]
pub struct Dropdown {
    /// Whether the dropdown is currently open
    pub is_open: bool,
    /// Currently selected value
    pub selected: String,
}

impl Default for Dropdown {
    fn default() -> Self {
        Self {
            is_open: false,
            selected: String::new(),
        }
    }
}

/// Component marking the trigger button of a dropdown
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
pub struct DropdownTrigger;

/// Component marking the text inside the dropdown trigger
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
pub struct DropdownTriggerText;

/// Component marking the options container (the list that appears)
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component, Default, Debug, PartialEq, Clone)]
pub struct DropdownOptions;

/// Component for individual dropdown options
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug, Clone)]
pub struct DropdownOption {
    /// The value of this option
    pub value: String,
}

/// Parameters for the dropdown template
pub struct DropdownProps {
    /// Width of the dropdown
    pub width: Val,
    /// Maximum height of the options list
    pub max_height: Val,
    /// Rounded corners options
    pub corners: RoundedCorners,
    /// Initial selected value (None shows "Select...")
    pub initial_value: Option<String>,
}

impl Default for DropdownProps {
    fn default() -> Self {
        Self {
            width: Val::Px(200.0),
            max_height: Val::Px(300.0),
            corners: RoundedCorners::default(),
            initial_value: None,
        }
    }
}

/// Template function to spawn a dropdown.
///
/// # Arguments
/// * `props` - construction properties for the dropdown.
/// * `overrides` - a bundle of components that are merged in.
/// * `options` - the dropdown options as a SpawnableList.
///
/// # Examples
/// ```ignore
/// dropdown(
///     DropdownProps::default(),
///     (),
///     Children::spawn(SpawnIter(
///         voices.iter().map(|v| dropdown_option(v))
///     ))
/// )
/// ```
pub fn dropdown<C: SpawnableList<ChildOf> + Send + Sync + 'static, B: Bundle>(
    props: DropdownProps,
    overrides: B,
    options: C,
) -> impl Bundle {
    let initial_text = props
        .initial_value
        .clone()
        .unwrap_or_else(|| "Select...".to_string());
    let selected = props.initial_value.clone().unwrap_or_default();

    (
        Node {
            width: props.width,
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Relative,
            ..Default::default()
        },
        Dropdown {
            is_open: false,
            selected,
        },
        overrides,
        children![
            // Trigger button
            (
                button(
                    ButtonProps {
                        corners: props.corners.clone(),
                        ..Default::default()
                    },
                    DropdownTrigger,
                    Spawn((Text::new(initial_text), ThemedText, DropdownTriggerText)),
                ),
                observe(on_trigger_click),
            ),
            // Options container (hidden by default)
            (
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0), // Position below trigger
                    left: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    display: Display::None, // Hidden by default
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                DropdownOptions,
                GlobalZIndex(1000),
                ThemeBackgroundColor(tokens::BUTTON_BG),
                children![scroll(
                    ScrollProps {
                        width: Val::Percent(100.0),
                        height: props.max_height,
                        overflow: bevy::ui::Overflow {
                            x: OverflowAxis::Visible,
                            y: OverflowAxis::Scroll,
                        },
                        flex_direction: FlexDirection::Column,
                        corners: props.corners,
                        bg_token: tokens::BUTTON_BG,
                        align_items: AlignItems::Stretch,
                    },
                    (),
                    Children::spawn(options),
                )],
            ),
        ],
    )
}

/// Helper function to create a dropdown option
pub fn dropdown_option(value: impl Into<String>) -> impl Bundle {
    let value = value.into();
    (
        button(
            ButtonProps::default(),
            (),
            Spawn((Text::new(value.clone()), ThemedText)),
        ),
        DropdownOption { value },
        observe(on_option_click),
    )
}

/// Observer that handles trigger button clicks to toggle dropdown
fn on_trigger_click(
    trigger: On<Pointer<Click>>,
    trigger_query: Query<&ChildOf, With<DropdownTrigger>>,
    mut dropdown_query: Query<&mut Dropdown>,
    child_of_query: Query<&ChildOf>,
) {

    // Find parent dropdown - may need to walk up hierarchy if Click bubbled from child
    let mut current = trigger.entity;

    // First try direct: is current entity the trigger?
    if let Ok(child_of) = trigger_query.get(current) {
        if let Ok(mut dropdown) = dropdown_query.get_mut(child_of.parent()) {
            dropdown.is_open = !dropdown.is_open;
            return;
        }
    }

    // Walk up hierarchy to find DropdownTrigger
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

/// Observer that handles option selection
fn on_option_click(
    trigger: On<Activate>,
    option_query: Query<(&DropdownOption, &ChildOf)>,
    mut dropdown_query: Query<(&mut Dropdown, &Children)>,
    child_of_query: Query<&ChildOf>,
    trigger_query: Query<&Children, With<DropdownTrigger>>,
    mut text_query: Query<&mut Text, With<DropdownTriggerText>>,
    mut commands: Commands,
) {
    let Ok((option, current_parent)) = option_query.get(trigger.entity) else {
        return;
    };

    // Walk up the hierarchy to find the dropdown
    let mut current = current_parent.parent();
    loop {
        if let Ok((mut dropdown, dropdown_children)) = dropdown_query.get_mut(current) {
            // Update selected value
            dropdown.selected = option.value.clone();

            // Close dropdown
            dropdown.is_open = false;

            // Find the trigger button in this specific dropdown's children
            for &child in dropdown_children {
                if let Ok(trigger_children) = trigger_query.get(child) {
                    // Find the text inside this trigger
                    for &text_child in trigger_children {
                        if let Ok(mut text) = text_query.get_mut(text_child) {
                            text.0 = option.value.clone();
                        }
                    }
                }
            }

            // Emit ValueChange event on the dropdown entity
            let value = option.value.clone();
            commands.entity(current).trigger(|entity| ValueChange {
                source: entity,
                value,
            });
            break;
        }

        // Try to go up one more level
        if let Ok(child_of) = child_of_query.get(current) {
            current = child_of.parent();
        } else {
            break;
        }
    }
}

/// System to update dropdown visibility based on is_open state
pub fn update_dropdown_visibility(
    changed_dropdowns: Query<(&Dropdown, &Children), Changed<Dropdown>>,
    trigger_query: Query<&DropdownTrigger>,
    mut style_query: Query<&mut Node, With<DropdownOptions>>,
) {
    for (dropdown, children) in changed_dropdowns.iter() {
        // Find the options container (it's the second child after the trigger)
        for &child in children {
            // Skip the trigger
            if trigger_query.get(child).is_ok() {
                continue;
            }

            // This should be the options container
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

