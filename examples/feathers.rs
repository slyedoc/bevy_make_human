//! Feathers UI example - demonstrates human_editor widget
#[path = "common/mod.rs"]
mod common;
pub use common::*;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    core_pipeline::tonemapping::Tonemapping::AcesFitted,
    feathers::{FeathersPlugins, controls::*, dark_theme::create_dark_theme, theme::*, tokens},
    input::common_conditions::input_toggle_active,
    picking::mesh_picking::MeshPickingPlugin,
    prelude::*,
    render::view::Hdr,
    ui_widgets::*,
};
use bevy_make_human::{
    prelude::*,
    ui::{HumanEditor, human_editor, text_input::handle_text_input_focus},
};
use bevy_ui_text_input::TextInputPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            FeathersPlugins,
            MeshPickingPlugin, // required for clicking on humans
            TextInputPlugin,   // required for text input fields in human editor
            // local
            MakeHumanPlugin::default(),
            CommonPlugin,
        ))
        .insert_resource(UiTheme(create_dark_theme()))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            animate_light.distributive_run_if(input_toggle_active(false, KeyCode::KeyL)),
        )
        .add_systems(
            Update,
            // stop camera controller when typing in text input
            handle_text_input_focus::<CameraFree>
                .run_if(resource_changed::<bevy::input_focus::InputFocus>),
        )
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Click on a human to open and close editor panel.");

    commands.spawn((
        Camera3d::default(),
        Hdr,
        AcesFitted,
        CameraFree::default(),
        Transform::from_xyz(0.0, 3., -5.0).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            ..default()
        })),
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        Transform::default(),
    ));

    commands
        .spawn((
            Name::new("Bob"),
            Human,
            Rig::Mixamo,
            SkinMesh::MaleGeneric,
            SkinMaterial::YoungCaucasianMale,
            Eyes::LowPolyBluegreen,
            Hair::CulturalibreHair02,
            Eyebrows::Eyebrow006,
            Eyelashes::Eyelashes01,
            Teeth::TeethBase,
            Tongue::Tongue01,
            Outfit(vec![
                Clothing::ToigoMaleSuit3,
                Clothing::ToigoAnkleBootsMale,
            ]),
            Morphs(vec![Morph::new(
                MorphTarget::Macro(MacroMorph::CaucasianMaleYoung),
                1.0,
            )]),
            Transform::from_xyz(-1.0, 0.0, 0.0),
        ))
        .observe(on_human_click);

    commands
        .spawn((
            Name::new("Sarah"),
            Human,
            Rig::Mixamo,
            SkinMesh::FemaleGeneric,
            SkinMaterial::YoungCaucasianFemale,
            Eyes::LowPolyBluegreen,
            Hair::ElvsLaraHair,
            Eyebrows::Eyebrow006,
            Eyelashes::Eyelashes04,
            Teeth::TeethBase,
            Tongue::Tongue01,
            Outfit(vec![Clothing::ElvsGoddessDress8]),
            Morphs(vec![Morph::new(
                MorphTarget::Macro(MacroMorph::CaucasianFemaleYoung),
                1.0,
            )]),
            Transform::from_xyz(1.0, 0.0, 0.0),
        ))
        .observe(on_human_click);
}

/// Moves the light around.
fn animate_light(
    mut lights: Query<&mut Transform, Or<(With<PointLight>, With<DirectionalLight>)>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for mut transform in lights.iter_mut() {
        transform.translation = vec3(
            ops::sin(now * 1.4),
            ops::cos(now * 1.0),
            ops::cos(now * 0.6),
        ) * vec3(3.0, 4.0, 3.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Container for editor panels
#[derive(Component)]
struct EditorPanels;

fn setup_ui(mut commands: Commands) {
    // Controls panel
    commands.spawn((
        Name::new("ControlsPanel"),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(20.0),
            left: px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(8.0)),
        children![
            (
                Text::new("Controls"),
                ThemedText,
                TextFont::from_font_size(14.0)
            ),
            (
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new("Add Human"), ThemedText))
                ),
                observe(spawn_new_character),
            ),
        ],
    ));

    // Container for editor panels (on the left side)
    commands.spawn((
        Name::new("EditorPanels"),
        EditorPanels,
        Node {
            position_type: PositionType::Absolute,
            top: px(20.0),
            left: px(20.0),
            bottom: px(100.0),
            flex_direction: FlexDirection::Row,
            column_gap: px(8.0),
            ..default()
        },
    ));
}

fn spawn_new_character(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    humans: Query<Entity, With<Human>>,
) {
    let count = humans.iter().count();
    let x = (count as f32) * 2.0;
    commands
        .spawn((
            Name::new(format!("Human_{}", count)),
            Human,
            Transform::from_xyz(x, 1.0, 0.0),
        ))
        .observe(on_human_click);
}

fn on_human_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    panels_container: Single<Entity, With<EditorPanels>>,
    existing_editors: Query<(Entity, &HumanEditor, &ChildOf)>,
) {
    let human = trigger.entity;

    // Check if editor already exists for this human
    for (_, widget, child_of) in existing_editors.iter() {
        if widget.0 == human {
            // Already has editor, remove wrapper parent (toggle off)
            commands.entity(child_of.parent()).despawn();
            return;
        }
    }

    // Spawn wrapper panel with close button and human_editor as child
    commands.entity(*panels_container).with_child((
        EditorPanel,
        Node {
            width: px(380.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(8.0)),
        children![
            // Close button wrapper
            (
                Node {
                    align_self: AlignSelf::FlexEnd,
                    margin: UiRect::new(px(0.0), px(4.0), px(4.0), px(0.0)),
                    ..default()
                },
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("x"), ThemedText))
                    ),
                    observe(close_editor_panel),
                )],
            ),
            // Human editor
            human_editor(human, ())
        ],
    ));
}

#[derive(Component)]
struct EditorPanel;

fn close_editor_panel(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    parents: Query<&ChildOf>,
    panels: Query<Entity, With<EditorPanel>>,
) {
    // find parent panel and despawn it
    for e in parents.iter_ancestors(trigger.entity) {
        if panels.contains(e) {
            commands.entity(e).despawn();
            return;
        }
    }
}
