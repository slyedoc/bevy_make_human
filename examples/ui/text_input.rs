use bevy::ecs::bundle::Bundle;
use bevy::feathers::{
    constants::size,
    rounded_corners::RoundedCorners,
    theme::{ThemeBackgroundColor, ThemeBorderColor, ThemeFontColor},
    tokens,
};
use bevy::input_focus::{InputFocus, tab_navigation::TabIndex};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{AlignItems, Display, FlexDirection, Node, UiRect, Val};

use bevy_enhanced_input::prelude::ContextActivity;
use bevy_ui_text_input::{
    TextInputBuffer, TextInputMode, TextInputNode, TextInputPrompt, TextInputStyle,
};

pub struct TextInputProps {
    pub width: Val,
    pub height: Val,
    pub placeholder: String,
    pub initial_text: String,
    pub corners: RoundedCorners,
    pub mode: TextInputMode,
    pub max_chars: Option<usize>,
}

impl Default for TextInputProps {
    fn default() -> Self {
        Self {
            width: Val::Px(280.0),
            height: size::ROW_HEIGHT,
            placeholder: String::new(),
            initial_text: String::new(),
            corners: RoundedCorners::All,
            mode: TextInputMode::SingleLine,
            max_chars: None,
        }
    }
}

// NOTE: BorderRadius removed due to 0.18.0-rc.2 bug (missing Component derive)
pub fn text_input<B: Bundle>(props: TextInputProps, overrides: B) -> impl Bundle {
    use cosmic_text::Edit;

    let mut buffer = TextInputBuffer::default();
    if !props.initial_text.is_empty() {
        buffer.editor.insert_string(&props.initial_text, None);
    }

    (
        Node {
            width: props.width,
            height: props.height,
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..Default::default()
        },
        TextInputNode {
            mode: props.mode,
            max_chars: props.max_chars,
            justification: Justify::Left,
            ..Default::default()
        },
        buffer,
        TextInputPrompt::new(props.placeholder),
        TextInputStyle::default(),
        Hovered::default(),
        TabIndex(0),
        ThemeBackgroundColor(tokens::BUTTON_BG),
        ThemeBorderColor(tokens::CHECKBOX_BORDER),
        ThemeFontColor(tokens::BUTTON_TEXT),
        overrides,
    )
}

/// System to enable/disable a bevy_enhanced_input context based on text input focus.
///
/// When a text input has focus, the context is disabled.
/// When focus is lost or moves to a non-text-input element, the context is enabled.
///
/// This is useful for preventing camera controls or other input actions from triggering
/// while typing in text fields.
///
/// # Usage
///
/// ```rust,ignore
/// app.add_systems(Update,
///     handle_text_input_focus::<CameraFree>.run_if(resource_changed::<InputFocus>)
/// );
/// ```
pub fn handle_text_input_focus<T>(
    input_focus: Res<InputFocus>,
    text_input_query: Query<&TextInputBuffer>,
    context_query: Query<Entity, With<T>>,
    mut commands: Commands,
) where
    T: Component,
{
    let Ok(context_entity) = context_query.single() else {
        return;
    };

    if let Some(focused_entity) = input_focus.0 {
        // Check if the focused entity is a text input
        if text_input_query.contains(focused_entity) {
            // Disable context when text input has focus
            commands
                .entity(context_entity)
                .insert(ContextActivity::<T>::INACTIVE);
        } else {
            // Enable context if focused entity is not a text input
            commands
                .entity(context_entity)
                .insert(ContextActivity::<T>::ACTIVE);
        }
    } else {
        // Enable context when nothing has focus
        commands
            .entity(context_entity)
            .insert(ContextActivity::<T>::ACTIVE);
    }
}
