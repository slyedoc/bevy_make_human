//! Raven Editor Plugin
//! Egui and Gizmo toggles for the most part
use avian3d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin},
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    gizmos::{
        aabb::AabbGizmoConfigGroup,
        config::{GizmoConfig, GizmoConfigStore},
        light::{LightGizmoColor, LightGizmoConfigGroup},
    },
    input::common_conditions::input_just_pressed,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    picking::pointer::{PointerId, PointerInteraction},
    prelude::*,
    render::diagnostic::RenderDiagnosticsPlugin,
    window::Monitor,
};
use bevy_egui::{EguiContext, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext, egui};
use bevy_enhanced_input::prelude::*;
use bevy_inspector_egui::{
    DefaultInspectorConfigPlugin,
    bevy_inspector::{hierarchy::SelectedEntities, ui_for_all_assets, ui_for_resources}, quick::StateInspectorPlugin,
};
#[allow(unused_imports)]
use bevy_make_human::prelude::*;
use std::ops::DerefMut;

pub mod prelude {
    pub use crate::{EditorPlugin, EditorState};
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum EditorState {
    Enabled,
    #[default]
    Disabled,
}

#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin::default(),
            DefaultInspectorConfigPlugin,
            StateInspectorPlugin::<MHState>::new()
                .run_if(in_state(EditorState::Enabled)),
            
            PhysicsDebugPlugin,
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin::default(),
            DebugPickingPlugin,
            // Diagnostics
            EntityCountDiagnosticsPlugin::default(),
            SystemInformationDiagnosticsPlugin::default(),
            RenderDiagnosticsPlugin,
        ))
        .init_state::<EditorState>()
        .insert_gizmo_config(
            LightGizmoConfigGroup {
                draw_all: true,
                color: LightGizmoColor::MatchLightColor,
                ..default()
            },
            GizmoConfig {
                enabled: false,
                ..default()
            },
        )
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, setup)
        .add_systems(Startup, || {
            info!("Press F1-F5 to toggle various editor features");
        })
        .add_systems(
            EguiPrimaryContextPass,
            inspector_ui.run_if(in_state(EditorState::Enabled)),
        )
        .add_systems(
            PreUpdate,
            (
                toggle_editor.run_if(input_just_pressed(KeyCode::F1)),
                (
                    toggle_physics,
                    toggle_lighting,
                    toggle_picking_debug,
                    
                )
                    .distributive_run_if(input_just_pressed(KeyCode::F2)),
                (
                    toggle_ui_debug,
                    toggle_aabb,
                    #[cfg(not(target_arch = "wasm32"))]
                    toggle_wireframe,
                    
                )
                    .distributive_run_if(input_just_pressed(KeyCode::F3)),                                            
                #[cfg(feature = "debug_draw")]
                toggle_skeleton.run_if(input_just_pressed(KeyCode::F4)),                
                #[cfg(feature = "debug_draw")]
                toggle_joint_axes.run_if(input_just_pressed(KeyCode::F5)),
            ),
        );
        
    }

    fn finish(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
    }
}

fn setup(
    mut config_store: ResMut<GizmoConfigStore>,
    state: Res<State<EditorState>>,
    mut ui_debug: ResMut<UiDebugOptions>,
    mut pick_debug: ResMut<DebugPickingMode>,
) {
    let enabled = match state.get() {
        EditorState::Enabled => true,
        EditorState::Disabled => false,
    };

    // aabb
    {
        let (store, aabb) = config_store.config_mut::<AabbGizmoConfigGroup>();
        // enable AABB gizmos for all entities, not relying on ShowAabb component
        aabb.draw_all = true;
        store.enabled = enabled;
    }

    // ui
    ui_debug.enabled = enabled;
    *pick_debug = match enabled {
        true => DebugPickingMode::Normal,
        false => DebugPickingMode::Disabled,
    };

    // avian
    {
        let config = config_store.config_mut::<PhysicsGizmos>().0;
        config.enabled = enabled;
    }

    // skeleton
    #[cfg(feature = "debug_draw")]
    {
        let config = config_store.config_mut::<SkeletonGizmos>().0;
        config.enabled = enabled;
    }
}

fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let Ok(mut ctx) = world
        .query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>()
        .single_mut(world)
    else {
        return;
    };
    let mut egui_context = ctx.deref_mut().clone();

    egui::SidePanel::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Entities");
                egui::CollapsingHeader::new("World")
                    .default_open(true)
                    .show(ui, |ui| {
                        bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui_filtered::<(
                            // ignore ( update Other if you add to ignore list)
                            Without<BindingOf>,    // bevy_enhanced_input
                            Without<ActionEvents>, // bevy_enhanced_input
                            //Without<GlobalRng>,    // rng
                            // seperate ui
                            Without<UiTransform>,
                            Without<PointerId>,
                            Without<Window>,
                            Without<Monitor>,
                        )>(world, ui, &mut selected_entities);
                    });

                egui::CollapsingHeader::new("UI").show(ui, |ui| {
                    bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui_filtered::<(
                        // ui
                        Or<(
                            With<UiTransform>,
                            With<PointerId>,
                            With<Window>,                            
                            With<Monitor>,
                        )>,
                    )>(world, ui, &mut selected_entities);
                });

                egui::CollapsingHeader::new("Other").show(ui, |ui| {
                    bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui_filtered::<
                        Or<(
                            With<BindingOf>,    // bevy_enhanced_input
                            With<ActionEvents>, // bevy_enhanced_input
                            // With<GlobalRng>,    // rng
                        )>,
                    >(world, ui, &mut selected_entities);
                });

                ui.heading("Data");

                egui::CollapsingHeader::new("Resources").show(ui, |ui| {
                    ui_for_resources(world, ui);
                });
                egui::CollapsingHeader::new("Assets").show(ui, |ui| {
                    ui_for_all_assets(world, ui);
                });
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world, entities, ui,
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}

pub fn in_editor(state: Res<State<EditorState>>) -> bool {
    match state.get() {
        EditorState::Enabled => true,
        EditorState::Disabled => false,
    }
}

fn toggle_editor(mut next_state: ResMut<NextState<EditorState>>, state: Res<State<EditorState>>) {
    next_state.set(match state.get() {
        EditorState::Enabled => EditorState::Disabled,
        EditorState::Disabled => EditorState::Enabled,
    });
}

fn toggle_aabb(mut config_store: ResMut<GizmoConfigStore>) {
    let (store, _aabb) = config_store.config_mut::<AabbGizmoConfigGroup>();
    store.enabled = !store.enabled;
}

fn toggle_physics(mut config_store: ResMut<GizmoConfigStore>) {
    let (store, _physics) = config_store.config_mut::<PhysicsGizmos>();
    store.enabled = !store.enabled;
}

#[cfg(feature = "debug_draw")]
fn toggle_skeleton(mut config_store: ResMut<GizmoConfigStore>) {
    let (store, _skeleton) = config_store.config_mut::<SkeletonGizmos>();
    store.enabled = !store.enabled;
}

#[cfg(feature = "debug_draw")]
fn toggle_joint_axes(mut config_store: ResMut<GizmoConfigStore>) {
    let (store, _axes) = config_store.config_mut::<JointAxesGizmos>();
    store.enabled = !store.enabled;
}

fn toggle_lighting(mut config_store: ResMut<GizmoConfigStore>) {
    let config = config_store.config_mut::<LightGizmoConfigGroup>().0;
    config.enabled = !config.enabled;
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(mut config: ResMut<WireframeConfig>) {
    config.global = !config.global;
}

fn toggle_picking_debug(mut mode: ResMut<DebugPickingMode>) {
    *mode = match *mode {
        DebugPickingMode::Disabled => DebugPickingMode::Normal,
        _ => DebugPickingMode::Disabled,
    };
}

fn toggle_ui_debug(mut ui: ResMut<UiDebugOptions>) {
    ui.enabled = !ui.enabled;
}

pub fn toggle_diagnostics_ui(mut settings: ResMut<PhysicsDiagnosticsUiSettings>) {
    settings.enabled = !settings.enabled;
}

#[allow(dead_code)]
fn toggle_atmspheric_fog(mut fog: Single<&mut DistanceFog>) {
    let a = fog.color.alpha();
    fog.color.set_alpha(1.0 - a);
}

#[allow(dead_code)]
fn toggle_directional_light_atmspheric_fog_influence(mut fog: Single<&mut DistanceFog>) {
    let a = fog.directional_light_color.alpha();
    fog.directional_light_color.set_alpha(0.5 - a);
}

#[allow(dead_code)]
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, tailwind::RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, tailwind::PINK_100);
    }
}

// /// Draw local axes (RGB = XYZ convention)
// fn draw_directions(query: Query<&GlobalTransform, With<DrawDirections>>, mut gizmos: Gizmos) {
//     for transform in query.iter() {
//         let pos = transform.translation();
//         let rot = transform.to_scale_rotation_translation().1;

//         // RGB = XYZ convention
//         gizmos.line(pos, pos + rot * Vec3::X, css::RED); // X = Red
//         gizmos.line(pos, pos + rot * Vec3::Y, css::GREEN); // Y = Green
//         gizmos.line(pos, pos + rot * Vec3::Z, css::BLUE); // Z = Blue
//     }
// }

// #[cfg(feature = "avian3d")]
// pub fn physics_paused(time: Res<Time<Physics>>) -> bool {
//     time.is_paused()
// }

// #[cfg(feature = "avian3d")]
// pub fn toggle_paused(mut time: ResMut<Time<Physics>>) {
//     if time.is_paused() {
//         time.unpause();
//     } else {
//         time.pause();
//     }
// }

// #[cfg(feature = "avian3d")]
// /// Advances the physics simulation by one `Time<Fixed>` time step.
// pub fn step(mut physics_time: ResMut<Time<Physics>>, fixed_time: Res<Time<Fixed>>) {
//     physics_time.advance_by(fixed_time.delta());
// }

