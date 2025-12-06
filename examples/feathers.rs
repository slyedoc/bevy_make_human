#[path ="common/mod.rs"]
mod common;
pub use common::*;

use std::f32::consts::PI;
use std::any::TypeId;
use bevy::{
    animation::{AnimationTargetId, animated_field, animation_curves::{EvaluatorId, AnimatableCurve, AnimatableKeyframeCurve}},
    app::AppExit,
    color::palettes::css,
    feathers::{controls::*, theme::*, tokens},
    gltf::{Gltf, GltfLoaderSettings},
    math::curve::{Interval, UnevenSampleAutoCurve, cores::UnevenCoreError},
    mesh::skinning::SkinnedMesh,
    picking::mesh_picking::MeshPickingPlugin,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    ui::{Checked, OverflowAxis},
    ui_widgets::*,
};
use bevy_asset_loader::prelude::*;
use bevy_mod_billboard::prelude::*;
use avian3d::prelude::*;
use bevy_make_human::prelude::*;
use bevy_inspector_egui::quick::StateInspectorPlugin;


/// Marker for the GLTF skeleton reference entity
#[derive(Component)]
struct GltfSkeletonRef;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Ready,
}

/// Currently selected human entity
#[derive(Resource, Default)]
struct SelectedHuman(Option<Entity>);

/// Marker for the config panel
#[derive(Component)]
struct ConfigPanel;

/// Marker for dropdown button showing current value
#[derive(Component)]
struct DropdownButton<T: 'static + Send + Sync>(std::marker::PhantomData<T>);

impl<T: 'static + Send + Sync> Default for DropdownButton<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Marker for dropdown popup
#[derive(Component)]
struct DropdownPopup<T: 'static + Send + Sync>(std::marker::PhantomData<T>);

/// Marker for dropdown option with value
#[derive(Component)]
struct DropdownOption<T: Clone + 'static + Send + Sync>(T);

/// Marker for the label text in dropdown button
#[derive(Component)]
struct DropdownLabel<T: 'static + Send + Sync>(std::marker::PhantomData<T>);

impl<T: 'static + Send + Sync> Default for DropdownLabel<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Marker for dropdown filter input
#[derive(Component)]
struct DropdownFilterInput<T: 'static + Send + Sync>(std::marker::PhantomData<T>);

impl<T: 'static + Send + Sync> Default for DropdownFilterInput<T> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

/// Marker for filterable dropdown item with lowercase name
#[derive(Component)]
struct DropdownFilterItem<T: 'static + Send + Sync>(String, std::marker::PhantomData<T>);

/// Marker for clothing list container
#[derive(Component)]
struct ClothingList;

/// Marker for individual clothing item with index
#[derive(Component)]
struct ClothingItem(usize);

/// Marker for add clothing dropdown
#[derive(Component)]
struct AddClothingDropdown;

/// Marker for phenotype slider with field name
#[derive(Component)]
struct PhenotypeSlider(&'static str);

/// Collapsible section state
#[derive(Component)]
struct CollapsibleSection {
    label: &'static str,
    collapsed: bool,
}

/// Track collapsed state across panel rebuilds
#[derive(Resource, Default)]
struct SectionStates {
    states: bevy::platform::collections::HashMap<&'static str, bool>,
}

/// Track structural state to detect add/remove (not value changes)
#[derive(Resource, Default)]
struct PanelState {
    morph_count: usize,
    clothing_count: usize,
}

/// Marker for section content that can be shown/hidden
#[derive(Component)]
struct SectionContent;

/// Marker for section header button
#[derive(Component)]
struct SectionHeader;

/// Marker for collapse indicator text (+/-)
#[derive(Component)]
struct CollapseIndicator;



fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // for faster load times, requires: "bevy/asset_processor",
            // .set(AssetPlugin {  
            //     mode: AssetMode::Processed,
            //     ..default()
            // }),
            MeshPickingPlugin,
            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
            CommonPlugin, // camera and egui editor
            
            //#[cfg(feature = "dev")] StateInspectorPlugin::<MHState>::default(),
        ))
        .init_state::<GameState>()
        .init_resource::<SelectedHuman>()
        .init_resource::<SectionStates>()
        .init_resource::<PanelState>()
        // .init_collection::<DipAssets>() // testing dip generated animations
        // .init_collection::<MixamoAssets>() // mixamo fbx animations
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .load_collection::<GltfAnimationAssets>()
                .continue_to_state(GameState::Ready),
        )
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(OnEnter(GameState::Ready), setup_gltf_skeleton_reference)
        .add_systems(
            Update,
            (control_animation_playback, update_play_button_text)
                .run_if(in_state(GameState::Ready)),
        )
        // Reset Armature scale after animation (Mixamo uses 0.01 scale)
        // .add_systems(
        //     PostUpdate,
        //     reset_armature_scale.after(bevy::animation::animate_targets),
        // )
        .add_systems(
            Update,
            (
                paint_joint_labels,
                setup_gltf_animations.run_if(in_state(GameState::Ready)),
                draw_joint_axes,
                update_config_panel,
                filter_clothing_options,
                filter_morph_options,
                filter_pose_options,
                load_pose_system,
                apply_pose_system,
                filter_dropdown_options::<SkinAsset>,
                filter_dropdown_options::<RigAsset>,
                filter_dropdown_options::<Option<HairAsset>>,
                filter_dropdown_options::<ProxyMesh>,
                filter_dropdown_options::<EyesAsset>,
                filter_dropdown_options::<EyeMaterialAsset>,
                filter_dropdown_options::<EyebrowsAsset>,
                filter_dropdown_options::<EyelashesAsset>,
                filter_dropdown_options::<TeethAsset>,
                filter_dropdown_options::<TongueAsset>,
            ),
        )
        // testing different animations systems
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        RavenCamera3d,
        CameraFree::default(),
        Transform::from_xyz(0.0, 3., -5.0).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y),
    ));

    // Lighting
    commands.spawn((
        dir_light(),
        Transform::from_xyz(4.0, 8.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.3, 0.35),
            ..default()
        })),
        Collider::half_space(Vec3::Y),
        RigidBody::Static,
        RayMesh,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // commands
    //     .spawn((
    //         Name::new("Bob"),
    //         Human,
    //         HumanConfig {
    //             proxy_mesh: ProxyMesh::MaleGeneric,
    //             rig: RigAsset::Maximo,
    //             skin: SkinAsset::YoungCaucasianMale,
    //             eyes: EyesAsset::LowPoly,
    //             eye_material: EyeMaterialAsset::Bluegreen,
    //             hair: Some(HairAsset::CulturalibreHair02),
    //             clothing: vec![
    //                 ClothingAsset::ToigoMaleSuit3,
    //                 ClothingAsset::ToigoAnkleBootsMale,
    //             ],
    //             morphs: vec![],
    //             ..default()
    //         },
    //         Phenotype {
    //             gender: 1.0,
    //             age: 0.5,
    //             muscle: 0.3,
    //             weight: 0.4,
    //             ..default()
    //         },
    //         RayMesh,
    //         DrawDirections,
    //         Transform::from_xyz(0.0, 0.0, 0.0),
    //     ))
    //     .observe(on_human_click)

    // Human is spawned in setup_human after GLTF assets are loaded
    // so we can extract reference skeleton rotations
}

fn setup_gltf_skeleton_reference(
    mut commands: Commands,
    gltf_assets: Res<GltfAnimationAssets>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<bevy::gltf::GltfNode>>,
) {
    let gltf = gltfs.get(&gltf_assets.breathing_idle).unwrap();
    // Spawn the GLTF skeleton scene for comparison
    commands.spawn((
        Name::new("GLTF_Breathing_Idle"),
        GltfSkeletonRef,
        SceneRoot(gltf.scenes[0].clone()),
        Transform::from_xyz(2.0, 0.0, 0.0), // Further left
    ));

    // Spawn human - base rotations now loaded automatically from mixamo.glb
    // when using RigAsset::Mixamo (via skeleton_glb_path())
    commands
        .spawn((
            Name::new("Sarah"),
            Human,
            Rig::Mixamo,
            Skin {
                mesh: Some(SkinMesh::FemaleGeneric),
                material: SkinMaterial::YoungCaucasianFemale,
            },
            Eyes {
                mesh: EyesMesh::LowPoly,
                material: EyesMaterial::Brown,
            },
            Hair::GrinsegoldWigBowTie,
            Eyebrows(EyebrowsAsset::Eyebrow006),
            Eyelashes(EyelashesAsset::Eyelashes02),
            Teeth(TeethAsset::TeethBase),
            Tongue(TongueAsset::Tongue01),
            Clothing(vec![ClothingAsset::ElvsSarongCoverUp]),
            Morphs::default(),
            Phenotype {
                gender: 0.0,
                age: 0.5,
                muscle: 0.5,
                weight: 0.4,
                ..default()
            },
            RayMesh,
            DrawDirections,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .observe(on_human_click)
        .observe(apply_gltf_animation);
}

/// Marker for entities that should have axis gizmos drawn
#[derive(Component)]
struct DrawJointAxes;

/// Mark GLTF skeleton joints for axis drawing and add text labels
fn paint_joint_labels(
    mut commands: Commands,
    skeleton_query: Query<(Entity, &Name), (With<GltfSkeletonRef>, Without<DrawJointAxes>)>,
    children_query: Query<&Children>,
    mesh_query: Query<&Mesh3d>,
    global_transforms: Query<&GlobalTransform>,
    names: Query<&Name>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");

    for (skeleton_entity, skeleton_name) in &skeleton_query {
        // Check if scene is loaded (has children)
        let Ok(children) = children_query.get(skeleton_entity) else {
            continue;
        };
        if children.is_empty() {
            continue;
        }

        let mut labeled = 0;
        for entity in children_query.iter_descendants(skeleton_entity) {
            // Skip entities with meshes (only label bones/joints)
            if mesh_query.get(entity).is_ok() {
                continue;
            }
            if global_transforms.get(entity).is_err() {
                continue;
            }

            let name = names
                .get(entity)
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "?".to_string());

            // Mark joint for axis gizmo drawing
            commands.entity(entity).insert(DrawJointAxes);

            // Spawn billboard text label
            commands
                .spawn((
                    ChildOf(entity),
                    BillboardText::default(),
                    TextLayout::new_with_justify(Justify::Left),
                    Transform {
                        translation: Vec3::Y * 0.03,
                        scale: Vec3::splat(0.001),
                        ..default()
                    },
                ))
                .with_child((
                    TextSpan::new(name),
                    TextFont::from(font.clone()).with_font_size(16.0),
                    TextColor(css::WHITE.into()),
                ));

            labeled += 1;
        }

        // Mark skeleton root so we don't process again
        commands.entity(skeleton_entity).insert(DrawJointAxes);
        info!("Labeled {} joints on {}", labeled, skeleton_name);
    }
}

/// Draw local axes at each joint using gizmos (RGB = XYZ)
fn draw_joint_axes(joints: Query<&GlobalTransform, With<DrawJointAxes>>, mut gizmos: Gizmos) {
    let axis_len = 0.04;

    for transform in &joints {
        let pos = transform.translation();
        let rot = transform.to_scale_rotation_translation().1;

        // RGB = XYZ convention
        gizmos.line(pos, pos + rot * Vec3::X * axis_len, css::RED); // X = Red
        gizmos.line(pos, pos + rot * Vec3::Y * axis_len, css::GREEN); // Y = Green
        gizmos.line(pos, pos + rot * Vec3::Z * axis_len, css::BLUE); // Z = Blue
    }
}

/// Marker for GLTF skeletons that have animations set up
#[derive(Component)]
struct GltfAnimationSetup;

/// Stores the animation node index for GLTF players
#[derive(Component)]
struct GltfAnimationNode(AnimationNodeIndex);

/// Set up animation graphs for GLTF skeletons once AnimationPlayer is available
fn setup_gltf_animations(
    mut commands: Commands,
    assets: Res<GltfAnimationAssets>,
    gltfs: Res<Assets<Gltf>>,
    skeleton_query: Query<(Entity, &Name), (With<GltfSkeletonRef>, Without<GltfAnimationSetup>)>,
    children_query: Query<&Children>,
    player_query: Query<Entity, (With<AnimationPlayer>, Without<AnimationGraphHandle>)>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (skeleton_entity, name) in &skeleton_query {
        // Find AnimationPlayer in descendants
        let Some(player_entity) = children_query
            .iter_descendants(skeleton_entity)
            .find(|e| player_query.get(*e).is_ok())
        else {
            continue;
        };

        // Determine which GLTF to use based on name
        let gltf_handle = if name.as_str().contains("Breathing") {
            &assets.breathing_idle
        } else {
            &assets.main_skeleton
        };

        let Some(gltf) = gltfs.get(gltf_handle) else {
            continue;
        };
        if gltf.animations.is_empty() {
            info!("{}: No animations in GLTF", name);
            commands.entity(skeleton_entity).insert(GltfAnimationSetup);
            continue;
        }

        // Create animation graph from first clip
        let clip = gltf.animations[0].clone();
        let (graph, node_index) = AnimationGraph::from_clip(clip);
        let graph_handle = animation_graphs.add(graph);

        // Attach graph to player and set up animation (paused)
        commands.entity(player_entity).insert((
            AnimationGraphHandle(graph_handle),
            GltfAnimationNode(node_index),
        ));

        commands.entity(skeleton_entity).insert(GltfAnimationSetup);
        info!("{}: Animation graph set up", name);
    }
}

#[allow(dead_code)]
fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rnd: Single<&mut WyRand, With<GlobalRng>>,
) {
    let grid = commands
        .spawn((
            Name::new("CharacterGrid"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    for (id, pos) in grid_positions(uvec2(2, 2), vec2(2.0, 2.0)) {
        commands.spawn((
            ChildOf(grid),
            Name::new(format!("Character_{id}")),
            Transform::from_translation(pos),
            Human,
            Skin {
                mesh: Some(*SkinMesh::iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rnd)
                    .unwrap()),
                material: *SkinMaterial::iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rnd)
                    .unwrap(),
            },
            *Hair::iter()
                .collect::<Vec<_>>()
                .choose(&mut rnd)
                .unwrap(),
            Eyebrows(*EyebrowsAsset::iter()
                .collect::<Vec<_>>()
                .choose(&mut rnd)
                .unwrap()),
            Eyelashes(*EyelashesAsset::iter()
                .collect::<Vec<_>>()
                .choose(&mut rnd)
                .unwrap()),
            Teeth(*TeethAsset::iter()
                .collect::<Vec<_>>()
                .choose(&mut rnd)
                .unwrap()),
            Tongue(*TongueAsset::iter()
                .collect::<Vec<_>>()
                .choose(&mut rnd)
                .unwrap()),
            Clothing(vec![
                *ClothingAsset::iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rnd)
                    .unwrap(),
                *ClothingAsset::iter()
                    .collect::<Vec<_>>()
                    .choose(&mut rnd)
                    .unwrap(),
            ]),
            DrawDirections,
        ));
    }
}

#[derive(Component)]
struct PlayPause;

fn setup_ui(mut commands: Commands) {
    // Controls panel at bottom left
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
                TextFont::from_font_size(14.0),
            ),
            (
                Name::new("PlayPauseButton"),
                PlayPause,
                button(
                    ButtonProps {
                        variant: ButtonVariant::Primary,
                        ..default()
                    },
                    (),
                    Spawn((
                        Text::new("Play"),
                        ThemedText,
                        TextFont::from_font_size(16.0),
                    )),
                ),
                observe(handle_play_pause_button),
            ),
            (
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((
                        Text::new("+ Character"),
                        ThemedText,
                        TextFont::from_font_size(14.0)
                    )),
                ),
                observe(spawn_new_character),
            ),
        ],
    ));

    // Character config panel at top left (hidden until selection)
    commands.spawn((
        Name::new("ConfigPanel"),
        ConfigPanel,
        Node {
            position_type: PositionType::Absolute,
            top: px(20.0),
            left: px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::all(px(12.0)),
            min_width: px(260.0),
            overflow: Overflow::visible(), // allow dropdown popups to escape
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(8.0)),
        Visibility::Hidden,
    ));

    // Visibility panel at top right
    commands.spawn((
        Name::new("VisibilityPanel"),
        Node {
            position_type: PositionType::Absolute,
            right: px(20.0),
            bottom: px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(8.0),
            padding: UiRect::all(px(12.0)),
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(8.0)),
        children![
            (
                Text::new("Visibility"),
                ThemedText,
                TextFont::from_font_size(14.0),
            ),
            #[cfg(feature = "dev")]
            (
                checkbox((), Spawn((Text::new("Skeleton"), ThemedText))),
                observe(toggle_skeleton),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Skin"), ThemedText))),
                observe(handle_check_visibility::<SkinMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Hair"), ThemedText))),
                observe(handle_check_visibility::<HairMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Eyes"), ThemedText))),
                observe(handle_check_visibility::<EyesMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Teeth"), ThemedText))),
                observe(handle_check_visibility::<TeethMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Tongue"), ThemedText))),
                observe(handle_check_visibility::<TongueMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Eyebrows"), ThemedText))),
                observe(handle_check_visibility::<EyebrowsMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Eyelashes"), ThemedText))),
                observe(handle_check_visibility::<EyelashesMesh>),
            ),
            (
                checkbox(Checked, Spawn((Text::new("Clothes"), ThemedText))),
                observe(handle_check_visibility::<ClothesMesh>),
            ),
        ],
    ));
}

#[cfg(feature = "dev")]
fn toggle_skeleton(
    trigger: On<ValueChange<bool>>,
    mut commands: Commands,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let mut checkbox = commands.entity(trigger.source);
    if trigger.value {
        checkbox.insert(Checked);
    } else {
        checkbox.remove::<Checked>();
    }

    let (store, _skeleton) = config_store.config_mut::<SkeletonGizmos>();
    store.enabled = trigger.value;
}

// Control animation playback with keyboard
fn control_animation_playback(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut AnimationPlayer, Option<&GltfAnimationNode>)>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        toggle_all_animations(&mut player_query);
    }
}

// Handle play button clicks
fn handle_play_pause_button(
    _trigger: On<Pointer<Click>>,
    mut player_query: Query<(&mut AnimationPlayer, Option<&GltfAnimationNode>)>,
) {
    toggle_all_animations(&mut player_query);
}

fn toggle_all_animations(
    player_query: &mut Query<(&mut AnimationPlayer, Option<&GltfAnimationNode>)>,
) {
    for (mut player, node) in player_query.iter_mut() {
        let is_playing = !player.all_paused() && player.playing_animations().count() > 0;
        if is_playing {
            player.pause_all();
        } else if player.playing_animations().count() > 0 {
            player.resume_all();
        } else if let Some(GltfAnimationNode(node_index)) = node {
            // Start animation for first time
            player.play(*node_index).repeat();
        }
    }
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
            HumanConfig::default(),
            Phenotype::default(),
            RayMesh,
            DrawDirections,
            Transform::from_xyz(x, 1.0, 0.0),
        ))
        .observe(on_human_click);
}

// Update button text based on playback state
fn update_play_button_text(
    player_query: Query<&AnimationPlayer>,
    button_query: Query<&Children, With<PlayPause>>,
    mut text_query: Query<&mut Text>,
) {
    let Some(player) = player_query.iter().next() else {
        return;
    };

    let is_playing = !player.all_paused() && player.playing_animations().count() > 0;
    let button_text = if is_playing { "Pause" } else { "Play" };

    for children in button_query.iter() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = button_text.to_string();
            }
        }
    }
}

//
// Character Selection & Config Panel
//

fn on_human_click(
    trigger: On<Pointer<Click>>,
    mut selected: ResMut<SelectedHuman>,
    human_query: Query<Entity, With<Human>>,
    parent_query: Query<&ChildOf>,
) {
    if human_query.get(trigger.entity).is_ok() {
        selected.0 = Some(trigger.entity);
        info!("Selected human: {:?}", trigger.entity);
        return;
    }
    // Walk up hierarchy to find human ancestor if any
    for c in parent_query.iter_ancestors(trigger.entity) {
        if human_query.get(c).is_ok() {
            selected.0 = Some(c);
            info!("Selected human: {:?}", c);
            return;
        }
    }
    selected.0 = None;
}

fn update_config_panel(
    selected: Res<SelectedHuman>,
    mut commands: Commands,
    config_panel: Query<Entity, With<ConfigPanel>>,
    human_query: Query<(&Name, &HumanConfig, &Phenotype), With<Human>>,
    section_states: Res<SectionStates>,
    mut panel_state: ResMut<PanelState>,
) {
    // Check if structural changes occurred (add/remove, not value changes)
    let structural_change = selected.0.is_some_and(|e| {
        if let Ok((_, config, _)) = human_query.get(e) {
            let changed = config.morphs.len() != panel_state.morph_count
                || config.clothing.len() != panel_state.clothing_count;
            if changed {
                panel_state.morph_count = config.morphs.len();
                panel_state.clothing_count = config.clothing.len();
            }
            changed
        } else {
            false
        }
    });

    if !selected.is_changed() && !structural_change {
        return;
    }

    // Update counts on selection change
    if selected.is_changed() {
        if let Some(e) = selected.0 {
            if let Ok((_, config, _)) = human_query.get(e) {
                panel_state.morph_count = config.morphs.len();
                panel_state.clothing_count = config.clothing.len();
            }
        }
    }

    let Ok(panel_entity) = config_panel.single() else {
        return;
    };

    // Clear existing children
    commands.entity(panel_entity).despawn_children();

    match selected.0 {
        Some(human_entity) => {
            commands.entity(panel_entity).insert(Visibility::Visible);

            if let Ok((name, config, phenotype)) = human_query.get(human_entity) {
                // Clone config values for closures
                let proxy_mesh = config.proxy_mesh;
                let rig = config.rig;
                let skin = config.skin;
                let hair = config.hair;
                let eyes = config.eyes;
                let eye_material = config.eye_material;
                let eyebrows = config.eyebrows;
                let eyelashes = config.eyelashes;
                let teeth = config.teeth;
                let tongue = config.tongue;
                let clothing = config.clothing.clone();
                let clothing_offset = config.clothing_offset;
                let floor_offset = config.floor_offset;
                let morphs = config.morphs.clone();
                let phenotype = *phenotype;

                // Get collapsed states (default: Body/Clothing/Phenotype expanded, Face/Mouth collapsed)
                let body_collapsed = *section_states.states.get("Body").unwrap_or(&false);
                let face_collapsed = *section_states.states.get("Face").unwrap_or(&false);
                let mouth_collapsed = *section_states.states.get("Mouth").unwrap_or(&false);
                let clothing_collapsed = *section_states.states.get("Clothing").unwrap_or(&false);
                let phenotype_collapsed = *section_states.states.get("Phenotype").unwrap_or(&false);
                let morphs_collapsed = *section_states.states.get("Morphs").unwrap_or(&false);
                let pose_collapsed = *section_states.states.get("Pose").unwrap_or(&false);

                // Spawn config panel content
                commands.entity(panel_entity).with_children(|parent| {
                    // Title (outside scroll)
                    parent.spawn((
                        Text::new(format!("Configure: {}", name)),
                        ThemedText,
                        TextFont::from_font_size(14.0),
                    ));

                    // Scrollable content container
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(8.0),
                            min_height: Val::Vh(60.0), // ensure room for dropdown popups
                            max_height: Val::Vh(80.0),
                            overflow: Overflow {
                                x: OverflowAxis::Visible,
                                y: OverflowAxis::Scroll,
                            },
                            ..default()
                        },))
                        .with_children(|scroll_content| {
                            // Body section
                            spawn_collapsible_section(
                                scroll_content,
                                "Body",
                                body_collapsed,
                                |content| {
                                    spawn_dropdown::<ProxyMesh>(
                                        content,
                                        "Mesh",
                                        proxy_mesh,
                                        human_entity,
                                    );
                                    spawn_dropdown::<RigAsset>(content, "Rig", rig, human_entity);
                                    spawn_dropdown::<SkinAsset>(
                                        content,
                                        "Skin",
                                        skin,
                                        human_entity,
                                    );
                                    spawn_floor_offset_slider(content, floor_offset, human_entity);
                                },
                            );

                            // Face section
                            spawn_collapsible_section(
                                scroll_content,
                                "Face",
                                face_collapsed,
                                |content| {
                                    spawn_optional_dropdown::<HairAsset>(
                                        content,
                                        "Hair",
                                        hair,
                                        human_entity,
                                    );
                                    spawn_dropdown::<EyesAsset>(
                                        content,
                                        "Eyes",
                                        eyes,
                                        human_entity,
                                    );
                                    spawn_dropdown::<EyeMaterialAsset>(
                                        content,
                                        "Eye Color",
                                        eye_material,
                                        human_entity,
                                    );
                                    spawn_dropdown::<EyebrowsAsset>(
                                        content,
                                        "Eyebrows",
                                        eyebrows,
                                        human_entity,
                                    );
                                    spawn_dropdown::<EyelashesAsset>(
                                        content,
                                        "Eyelashes",
                                        eyelashes,
                                        human_entity,
                                    );
                                },
                            );

                            // Mouth section
                            spawn_collapsible_section(
                                scroll_content,
                                "Mouth",
                                mouth_collapsed,
                                |content| {
                                    spawn_dropdown::<TeethAsset>(
                                        content,
                                        "Teeth",
                                        teeth,
                                        human_entity,
                                    );
                                    spawn_dropdown::<TongueAsset>(
                                        content,
                                        "Tongue",
                                        tongue,
                                        human_entity,
                                    );
                                },
                            );

                            // Clothing section
                            spawn_collapsible_section(
                                scroll_content,
                                "Clothing",
                                clothing_collapsed,
                                |content| {
                                    spawn_clothing_list_content(
                                        content,
                                        &clothing,
                                        clothing_offset,
                                        human_entity,
                                    );
                                },
                            );

                            // Phenotype section
                            spawn_collapsible_section(
                                scroll_content,
                                "Phenotype",
                                phenotype_collapsed,
                                |content| {
                                    spawn_phenotype_sliders_content(
                                        content,
                                        &phenotype,
                                        human_entity,
                                    );
                                },
                            );

                            // Morphs section
                            spawn_collapsible_section(
                                scroll_content,
                                "Morphs",
                                morphs_collapsed,
                                |content| {
                                    spawn_morphs_list_content(content, &morphs, human_entity);
                                },
                            );

                            // Pose section
                            spawn_collapsible_section(
                                scroll_content,
                                "Pose",
                                pose_collapsed,
                                |content| {
                                    spawn_pose_list_content(content, human_entity);
                                },
                            );
                        });

                    // Close button (outside scroll)
                    parent.spawn((
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new("Deselect"), ThemedText)),
                        ),
                        observe(
                            |_: On<Pointer<Click>>, mut selected: ResMut<SelectedHuman>| {
                                selected.0 = None;
                            },
                        ),
                    ));
                });
            }
        }
        None => {
            commands.entity(panel_entity).insert(Visibility::Hidden);
        }
    }
}

/// Marker for target human entity
#[derive(Component)]
struct TargetHuman(Entity);

/// Helper to spawn a collapsible section
fn spawn_collapsible_section(
    parent: &mut ChildSpawnerCommands,
    label: &'static str,
    collapsed: bool,
    content_spawner: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(2.0),
                ..default()
            },
            CollapsibleSection { label, collapsed },
        ))
        .with_children(|section| {
            // Header button with collapse indicator
            section.spawn((
                SectionHeader,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(4.0),
                    padding: UiRect::axes(px(4.0), px(2.0)),
                    ..default()
                },
                bevy::prelude::Button,
                ThemeBackgroundColor(tokens::BUTTON_BG),
                observe(on_section_toggle),
                children![
                    (
                        CollapseIndicator,
                        Text::new(if collapsed { "+" } else { "-" }),
                        ThemedText,
                        TextFont::from_font_size(12.0),
                        Node {
                            width: px(12.0),
                            ..default()
                        },
                    ),
                    (Text::new(label), ThemedText, TextFont::from_font_size(12.0),),
                ],
            ));

            // Content container
            section
                .spawn((
                    SectionContent,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4.0),
                        padding: UiRect::left(px(8.0)),
                        display: if collapsed {
                            Display::None
                        } else {
                            Display::Flex
                        },
                        ..default()
                    },
                ))
                .with_children(|content| {
                    content_spawner(content);
                });
        });
}

/// Handler for section collapse/expand toggle
fn on_section_toggle(
    trigger: On<Pointer<Click>>,
    header_query: Query<&ChildOf, With<SectionHeader>>,
    mut section_query: Query<(&mut CollapsibleSection, &Children)>,
    mut content_query: Query<&mut Node, With<SectionContent>>,
    children_query: Query<&Children>,
    mut indicator_query: Query<&mut Text, With<CollapseIndicator>>,
    mut section_states: ResMut<SectionStates>,
) {
    // Get the header's parent (the section)
    let Ok(child_of) = header_query.get(trigger.target()) else {
        return;
    };
    let Ok((mut section, section_children)) = section_query.get_mut(child_of.parent()) else {
        return;
    };

    section.collapsed = !section.collapsed;

    // Save state to resource
    section_states
        .states
        .insert(section.label, section.collapsed);

    // Update content visibility
    for child in section_children.iter() {
        if let Ok(mut node) = content_query.get_mut(child) {
            node.display = if section.collapsed {
                Display::None
            } else {
                Display::Flex
            };
        }
    }

    // Update the indicator text by finding it in header's children
    if let Ok(header_children) = children_query.get(trigger.target()) {
        for child in header_children.iter() {
            if let Ok(mut text) = indicator_query.get_mut(child) {
                **text = if section.collapsed { "+" } else { "-" }.to_string();
            }
        }
    }
}

fn spawn_dropdown<T>(parent: &mut ChildSpawnerCommands, label: &str, current: T, target: Entity)
where
    T: IntoEnumIterator
        + std::fmt::Display
        + Clone
        + Copy
        + PartialEq
        + Send
        + Sync
        + ConfigField
        + HasThumbnail
        + 'static,
{
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            ..default()
        },
        children![
            (Text::new(label), ThemedText, TextFont::from_font_size(12.0),),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.0),
                    ..default()
                },
                TargetHuman(target),
                DropdownButton::<T>::default(),
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((
                            DropdownLabel::<T>::default(),
                            Text::new(format!("{}", current)),
                            ThemedText,
                            TextFont::from_font_size(12.0),
                        )),
                    ),
                    observe(toggle_dropdown::<T>),
                ),],
            ),
        ],
    ));
}

fn spawn_optional_dropdown<T>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    current: Option<T>,
    target: Entity,
) where
    T: IntoEnumIterator
        + std::fmt::Display
        + Clone
        + Copy
        + PartialEq
        + Send
        + Sync
        + HasThumbnail
        + 'static,
    Option<T>: ConfigField,
{
    let display = match &current {
        Some(v) => format!("{}", v),
        None => "None".to_string(),
    };

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            ..default()
        },
        children![
            (Text::new(label), ThemedText, TextFont::from_font_size(12.0),),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.0),
                    ..default()
                },
                TargetHuman(target),
                DropdownButton::<Option<T>>::default(),
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((
                            DropdownLabel::<Option<T>>::default(),
                            Text::new(display),
                            ThemedText,
                            TextFont::from_font_size(12.0),
                        )),
                    ),
                    observe(toggle_optional_dropdown::<T>),
                ),],
            ),
        ],
    ));
}

fn toggle_dropdown<T>(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dropdown_query: Query<(Entity, &TargetHuman), With<DropdownButton<T>>>,
    popup_query: Query<Entity, With<DropdownPopup<T>>>,
) where
    T: IntoEnumIterator
        + std::fmt::Display
        + Clone
        + Copy
        + PartialEq
        + Send
        + Sync
        + ConfigField
        + HasThumbnail
        + 'static,
{
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    // Collect variants with their thumbnail handles
    let items: Vec<_> = T::iter()
        .map(|v| {
            let thumb = v.thumbnail_path().map(|p| asset_server.load::<Image>(p));
            (v, thumb)
        })
        .collect();

    let human_entity = target.0;
    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                DropdownPopup::<T>(std::marker::PhantomData),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    width: px(220.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                // Filter input
                popup.spawn((
                    Node {
                        padding: UiRect::all(px(4.0)),
                        ..default()
                    },
                    children![(
                        DropdownFilterInput::<T>::default(),
                        text_input(
                            TextInputProps {
                                placeholder: "Filter...".into(),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ()
                        ),
                    )],
                ));
                // Scrollable list
                popup.spawn(scroll(
                    ScrollProps {
                        width: Val::Percent(100.0),
                        height: px(250.0),
                        overflow: Overflow {
                            x: OverflowAxis::Visible,
                            y: OverflowAxis::Scroll,
                        },
                        flex_direction: FlexDirection::Column,
                        corners: Default::default(),
                        bg_token: tokens::WINDOW_BG,
                        align_items: AlignItems::Stretch,
                    },
                    (),
                    Children::spawn(SpawnIter(items.into_iter().map(move |(variant, thumb)| {
                        let label = format!("{}", variant);
                        let filter_name = label.to_lowercase();
                        (
                            DropdownOption(variant),
                            DropdownFilterItem::<T>(filter_name, std::marker::PhantomData),
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                padding: UiRect::all(px(4.0)),
                                ..default()
                            },
                            bevy::prelude::Button,
                            ThemeBackgroundColor(tokens::BUTTON_BG),
                            observe(
                                move |_: On<Pointer<Click>>,
                                      mut commands: Commands,
                                      popup_query: Query<Entity, With<DropdownPopup<T>>>,
                                      mut human_query: Query<&mut HumanConfig>,
                                      mut label_query: Query<&mut Text, With<DropdownLabel<T>>>| {
                                    if let Ok(mut config) = human_query.get_mut(human_entity) {
                                        variant.apply(&mut config);
                                    }
                                    if let Ok(mut text) = label_query.single_mut() {
                                        **text = format!("{}", variant);
                                    }
                                    if let Ok(popup) = popup_query.single() {
                                        commands.entity(popup).despawn();
                                    }
                                },
                            ),
                            Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                                if let Some(handle) = thumb.clone() {
                                    spawner.spawn((
                                        ImageNode::new(handle),
                                        Node {
                                            width: px(24.0),
                                            height: px(24.0),
                                            ..default()
                                        },
                                    ));
                                }
                                spawner.spawn((
                                    Text::new(label.clone()),
                                    ThemedText,
                                    TextFont::from_font_size(11.0),
                                ));
                            })),
                        )
                    }))),
                ));
            });
    });
}

fn toggle_optional_dropdown<T>(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dropdown_query: Query<(Entity, &TargetHuman), With<DropdownButton<Option<T>>>>,
    popup_query: Query<Entity, With<DropdownPopup<Option<T>>>>,
) where
    T: IntoEnumIterator
        + std::fmt::Display
        + Clone
        + Copy
        + PartialEq
        + Send
        + Sync
        + HasThumbnail
        + 'static,
    Option<T>: ConfigField,
{
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    // Collect variants with their thumbnail handles (None option first)
    let items: Vec<_> = std::iter::once((None, None))
        .chain(T::iter().map(|v| {
            let thumb = v.thumbnail_path().map(|p| asset_server.load::<Image>(p));
            (Some(v), thumb)
        }))
        .collect();

    let human_entity = target.0;
    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                DropdownPopup::<Option<T>>(std::marker::PhantomData),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    width: px(220.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                // Filter input
                popup.spawn((
                    Node {
                        padding: UiRect::all(px(4.0)),
                        ..default()
                    },
                    children![(
                        DropdownFilterInput::<Option<T>>::default(),
                        text_input(
                            TextInputProps {
                                placeholder: "Filter...".into(),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ()
                        ),
                    )],
                ));
                // Scrollable list
                popup.spawn(scroll(
                    ScrollProps {
                        width: Val::Percent(100.0),
                        height: px(250.0),
                        overflow: Overflow {
                            x: OverflowAxis::Visible,
                            y: OverflowAxis::Scroll,
                        },
                        flex_direction: FlexDirection::Column,
                        corners: Default::default(),
                        bg_token: tokens::WINDOW_BG,
                        align_items: AlignItems::Stretch,
                    },
                    (),
                    Children::spawn(SpawnIter(items.into_iter().map(move |(variant, thumb)| {
                        let label = variant
                            .map(|v| format!("{}", v))
                            .unwrap_or_else(|| "None".to_string());
                        let filter_name = label.to_lowercase();
                        (
                            DropdownOption(variant),
                            DropdownFilterItem::<Option<T>>(filter_name, std::marker::PhantomData),
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                padding: UiRect::all(px(4.0)),
                                ..default()
                            },
                            bevy::prelude::Button,
                            ThemeBackgroundColor(tokens::BUTTON_BG),
                            observe(
                                move |_: On<Pointer<Click>>,
                                      mut commands: Commands,
                                      popup_query: Query<
                                    Entity,
                                    With<DropdownPopup<Option<T>>>,
                                >,
                                      mut human_query: Query<&mut HumanConfig>,
                                      mut label_query: Query<
                                    &mut Text,
                                    With<DropdownLabel<Option<T>>>,
                                >| {
                                    if let Ok(mut config) = human_query.get_mut(human_entity) {
                                        variant.apply(&mut config);
                                    }
                                    if let Ok(mut text) = label_query.single_mut() {
                                        **text = variant
                                            .map(|v| format!("{}", v))
                                            .unwrap_or_else(|| "None".to_string());
                                    }
                                    if let Ok(popup) = popup_query.single() {
                                        commands.entity(popup).despawn();
                                    }
                                },
                            ),
                            Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                                if let Some(handle) = thumb.clone() {
                                    spawner.spawn((
                                        ImageNode::new(handle),
                                        Node {
                                            width: px(24.0),
                                            height: px(24.0),
                                            ..default()
                                        },
                                    ));
                                }
                                spawner.spawn((
                                    Text::new(label.clone()),
                                    ThemedText,
                                    TextFont::from_font_size(11.0),
                                ));
                            })),
                        )
                    }))),
                ));
            });
    });
}

//
// Clothing List UI
//

/// Marker for clothing offset slider
#[derive(Component)]
struct ClothingOffsetSlider;

/// Marker for floor offset slider
#[derive(Component)]
struct FloorOffsetSlider;

fn spawn_floor_offset_slider(parent: &mut ChildSpawnerCommands, floor_offset: f32, target: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Floor Offset"),
                        ThemedText,
                        TextFont::from_font_size(10.0)
                    ),
                    (
                        Text::new(format!("{:.3}", floor_offset)),
                        ThemedText,
                        TextFont::from_font_size(10.0)
                    ),
                ],
            ),
            (
                FloorOffsetSlider,
                TargetHuman(target),
                slider(
                    SliderProps {
                        min: -0.1,
                        max: 0.1,
                        value: floor_offset,
                        ..default()
                    },
                    (SliderStep(0.001), SliderPrecision(3)),
                ),
                observe(on_floor_offset_change),
            ),
        ],
    ));
}

fn on_floor_offset_change(
    trigger: On<ValueChange<f32>>,
    slider_query: Query<&TargetHuman, With<FloorOffsetSlider>>,
    mut config_query: Query<&mut HumanConfig>,
    mut commands: Commands,
) {
    let Ok(target) = slider_query.get(trigger.source) else {
        return;
    };
    let Ok(mut config) = config_query.get_mut(target.0) else {
        return;
    };
    config.floor_offset = trigger.value;
    commands
        .entity(trigger.source)
        .insert(SliderValue(trigger.value));
}

/// Spawn just the clothing list content (for use inside collapsible section)
fn spawn_clothing_list_content(
    parent: &mut ChildSpawnerCommands,
    clothing: &[ClothingAsset],
    clothing_offset: f32,
    target: Entity,
) {
    // Offset slider at top
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (
                        Text::new("Offset"),
                        ThemedText,
                        TextFont::from_font_size(10.0)
                    ),
                    (
                        Text::new(format!("{:.4}", clothing_offset)),
                        ThemedText,
                        TextFont::from_font_size(10.0)
                    ),
                ],
            ),
            (
                ClothingOffsetSlider,
                TargetHuman(target),
                slider(
                    SliderProps {
                        min: 0.0,
                        max: 0.01,
                        value: clothing_offset,
                        ..default()
                    },
                    (SliderStep(0.0001), SliderPrecision(4)),
                ),
                observe(on_clothing_offset_change),
            ),
        ],
    ));

    parent
        .spawn((
            ClothingList,
            TargetHuman(target),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            },
        ))
        .with_children(|list| {
            // Current items with remove buttons
            for (idx, item) in clothing.iter().enumerate() {
                list.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ClothingItem(idx),
                    children![
                    (
                        Text::new(format!("{}", item)),
                        ThemedText,
                        TextFont::from_font_size(11.0),
                        Node { flex_grow: 1.0, ..default() },
                    ),
                    (
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new("X"), ThemedText, TextFont::from_font_size(10.0))),
                        ),
                        observe(move |_: On<Pointer<Click>>,
                                      mut human_query: Query<&mut HumanConfig>,
                                      list_query: Query<&TargetHuman, With<ClothingList>>| {
                            if let Ok(target) = list_query.single() {
                                if let Ok(mut config) = human_query.get_mut(target.0) {
                                    if idx < config.clothing.len() {
                                        config.clothing.remove(idx);
                                    }
                                }
                            }
                        }),
                    ),
                ],
                ));
            }

            // Add button with dropdown
            spawn_add_clothing_button(list, target);
        });
}

fn on_clothing_offset_change(
    trigger: On<ValueChange<f32>>,
    slider_query: Query<&TargetHuman, With<ClothingOffsetSlider>>,
    mut config_query: Query<&mut HumanConfig>,
    mut commands: Commands,
) {
    let Ok(target) = slider_query.get(trigger.source) else {
        return;
    };
    let Ok(mut config) = config_query.get_mut(target.0) else {
        return;
    };
    config.clothing_offset = trigger.value;
    commands
        .entity(trigger.source)
        .insert(SliderValue(trigger.value));
}

fn spawn_add_clothing_button(parent: &mut ChildSpawnerCommands, target: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(4.0),
            ..default()
        },
        AddClothingDropdown,
        TargetHuman(target),
        children![(
            button(
                ButtonProps::default(),
                (),
                Spawn((
                    Text::new("+ Add"),
                    ThemedText,
                    TextFont::from_font_size(11.0)
                )),
            ),
            observe(toggle_add_clothing_dropdown),
        ),],
    ));
}

#[derive(Component)]
struct AddClothingPopup;

/// Marker for clothing filter text input
#[derive(Component)]
struct ClothingFilterInput;

/// Marker for filterable clothing option with lowercase name
#[derive(Component)]
struct ClothingOptionItem(String);

/// Container for the scrollable clothing list
#[derive(Component)]
struct ClothingOptionsList;

fn toggle_add_clothing_dropdown(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dropdown_query: Query<(Entity, &TargetHuman), With<AddClothingDropdown>>,
    popup_query: Query<Entity, With<AddClothingPopup>>,
) {
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    // Collect variants with their thumbnail handles
    let items: Vec<_> = ClothingAsset::iter()
        .map(|v| {
            let thumb = v.thumbnail_path().map(|p| asset_server.load::<Image>(p));
            (v, thumb)
        })
        .collect();

    let human_entity = target.0;
    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                AddClothingPopup,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    width: px(250.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                // Filter text input wrapper
                popup.spawn((
                    Node {
                        padding: UiRect::all(px(4.0)),
                        ..default()
                    },
                    children![(
                        ClothingFilterInput,
                        text_input(
                            TextInputProps {
                                placeholder: "Filter...".into(),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ()
                        ),
                    )],
                ));

                // Scrollable list
                popup.spawn((
                    ClothingOptionsList,
                    scroll(
                        ScrollProps {
                            width: Val::Percent(100.0),
                            height: px(250.0),
                            overflow: Overflow {
                                x: OverflowAxis::Visible,
                                y: OverflowAxis::Scroll,
                            },
                            flex_direction: FlexDirection::Column,
                            corners: Default::default(),
                            bg_token: tokens::WINDOW_BG,
                            align_items: AlignItems::Stretch,
                        },
                        (),
                        Children::spawn(SpawnIter(items.into_iter().map(
                            move |(variant, thumb)| {
                                let label = format!("{}", variant);
                                let filter_name = label.to_lowercase();
                                (
                            ClothingOptionItem(filter_name),
                            Node {
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: px(6.0),
                                padding: UiRect::all(px(4.0)),
                                ..default()
                            },
                            bevy::prelude::Button,
                            ThemeBackgroundColor(tokens::BUTTON_BG),
                            observe(move |_: On<Pointer<Click>>,
                                          mut commands: Commands,
                                          popup_query: Query<Entity, With<AddClothingPopup>>,
                                          mut human_query: Query<&mut HumanConfig>| {
                                if let Ok(mut config) = human_query.get_mut(human_entity) {
                                    config.clothing.push(variant);
                                }
                                if let Ok(popup) = popup_query.single() {
                                    commands.entity(popup).despawn();
                                }
                            }),
                            Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                                if let Some(handle) = thumb.clone() {
                                    spawner.spawn((
                                        ImageNode::new(handle),
                                        Node {
                                            width: px(24.0),
                                            height: px(24.0),
                                            ..default()
                                        },
                                    ));
                                }
                                spawner.spawn((
                                    Text::new(label.clone()),
                                    ThemedText,
                                    TextFont::from_font_size(11.0),
                                ));
                            })),
                        )
                            },
                        ))),
                    ),
                ));
            });
    });
}

/// Filter clothing options based on text input
fn filter_clothing_options(
    filter_query: Query<&TextInputBuffer, (With<ClothingFilterInput>, Changed<TextInputBuffer>)>,
    mut items_query: Query<(&ClothingOptionItem, &mut Node)>,
) {
    let Ok(buffer) = filter_query.single() else {
        return;
    };
    let filter = buffer.get_text().to_lowercase();

    for (item, mut node) in items_query.iter_mut() {
        node.display = if filter.is_empty() || item.0.contains(&filter) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn filter_dropdown_options<T: 'static + Send + Sync>(
    filter_query: Query<&TextInputBuffer, (With<DropdownFilterInput<T>>, Changed<TextInputBuffer>)>,
    mut items_query: Query<(&DropdownFilterItem<T>, &mut Node)>,
) {
    let Ok(buffer) = filter_query.single() else {
        return;
    };
    let filter = buffer.get_text().to_lowercase();

    for (item, mut node) in items_query.iter_mut() {
        node.display = if filter.is_empty() || item.0.contains(&filter) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

//
// Phenotype Sliders
//

#[derive(Component)]
struct RaceDropdownButton;

#[derive(Component)]
struct RaceDropdownPopup;

#[derive(Component)]
struct RaceDropdownLabel;

fn spawn_race_dropdown(parent: &mut ChildSpawnerCommands, current: Race, target: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            ..default()
        },
        children![
            (
                Text::new("Race"),
                ThemedText,
                TextFont::from_font_size(12.0)
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4.0),
                    ..default()
                },
                TargetHuman(target),
                RaceDropdownButton,
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((
                            RaceDropdownLabel,
                            Text::new(format!("{}", current)),
                            ThemedText,
                            TextFont::from_font_size(12.0),
                        )),
                    ),
                    observe(toggle_race_dropdown),
                )],
            ),
        ],
    ));
}

fn toggle_race_dropdown(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    dropdown_query: Query<(Entity, &TargetHuman), With<RaceDropdownButton>>,
    popup_query: Query<Entity, With<RaceDropdownPopup>>,
) {
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    let human_entity = target.0;
    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                RaceDropdownPopup,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(px(4.0)),
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                for variant in Race::iter() {
                    let label = format!("{}", variant);
                    popup.spawn(
                        (
                            Node {
                                padding: UiRect::all(px(4.0)),
                                ..default()
                            },
                            bevy::prelude::Button,
                            ThemeBackgroundColor(tokens::BUTTON_BG),
                            observe(
                                move |_: On<Pointer<Click>>,
                                      mut commands: Commands,
                                      popup_query: Query<Entity, With<RaceDropdownPopup>>,
                                      mut phenotype_query: Query<&mut Phenotype>,
                                      mut label_query: Query<
                                    &mut Text,
                                    With<RaceDropdownLabel>,
                                >| {
                                    if let Ok(mut phenotype) = phenotype_query.get_mut(human_entity)
                                    {
                                        phenotype.race = variant;
                                    }
                                    if let Ok(mut text) = label_query.single_mut() {
                                        **text = format!("{}", variant);
                                    }
                                    if let Ok(popup) = popup_query.single() {
                                        commands.entity(popup).despawn();
                                    }
                                },
                            ),
                            children![(
                                Text::new(label),
                                ThemedText,
                                TextFont::from_font_size(11.0)
                            )],
                        ),
                    );
                }
            });
    });
}

/// Spawn just the phenotype sliders content (for use inside collapsible section)
fn spawn_phenotype_sliders_content(
    parent: &mut ChildSpawnerCommands,
    phenotype: &Phenotype,
    target: Entity,
) {
    spawn_race_dropdown(parent, phenotype.race, target);
    spawn_phenotype_slider(parent, "Gender", phenotype.gender, target, "gender");
    spawn_phenotype_slider(parent, "Age", phenotype.age, target, "age");
    spawn_phenotype_slider(parent, "Muscle", phenotype.muscle, target, "muscle");
    spawn_phenotype_slider(parent, "Weight", phenotype.weight, target, "weight");
    spawn_phenotype_slider(parent, "Height", phenotype.height, target, "height");
    spawn_phenotype_slider(
        parent,
        "Proportions",
        phenotype.proportions,
        target,
        "proportions",
    );
    spawn_phenotype_slider(parent, "Cup Size", phenotype.cupsize, target, "cupsize");
    spawn_phenotype_slider(parent, "Firmness", phenotype.firmness, target, "firmness");
}

fn spawn_phenotype_slider(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: f32,
    target: Entity,
    field: &'static str,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2.0),
            ..default()
        },
        children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (Text::new(label), ThemedText, TextFont::from_font_size(10.0)),
                    (
                        Text::new(format!("{:.2}", value)),
                        ThemedText,
                        TextFont::from_font_size(10.0)
                    ),
                ],
            ),
            (
                PhenotypeSlider(field),
                TargetHuman(target),
                slider(
                    SliderProps {
                        min: 0.0,
                        max: 1.0,
                        value,
                        ..default()
                    },
                    (SliderStep(0.01), SliderPrecision(2)),
                ),
                observe(on_phenotype_slider_change),
            ),
        ],
    ));
}

fn on_phenotype_slider_change(
    trigger: On<ValueChange<f32>>,
    slider_query: Query<(&PhenotypeSlider, &TargetHuman)>,
    mut phenotype_query: Query<&mut Phenotype>,
    mut commands: Commands,
) {
    let Ok((field, target)) = slider_query.get(trigger.source) else {
        return;
    };
    let Ok(mut phenotype) = phenotype_query.get_mut(target.0) else {
        return;
    };

    match field.0 {
        "gender" => phenotype.gender = trigger.value,
        "age" => phenotype.age = trigger.value,
        "muscle" => phenotype.muscle = trigger.value,
        "weight" => phenotype.weight = trigger.value,
        "height" => phenotype.height = trigger.value,
        "proportions" => phenotype.proportions = trigger.value,
        "cupsize" => phenotype.cupsize = trigger.value,
        "firmness" => phenotype.firmness = trigger.value,
        _ => {}
    }

    // Update slider value
    commands
        .entity(trigger.source)
        .insert(SliderValue(trigger.value));
}

/// Trait for types that can update a HumanConfig field
trait ConfigField: Sized {
    fn apply(self, config: &mut HumanConfig);
}

impl ConfigField for ProxyMesh {
    fn apply(self, config: &mut HumanConfig) {
        config.proxy_mesh = self;
    }
}
impl ConfigField for SkinAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.skin = self;
    }
}
impl ConfigField for RigAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.rig = self;
    }
}
impl ConfigField for EyesAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.eyes = self;
    }
}
impl ConfigField for EyeMaterialAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.eye_material = self;
    }
}
impl ConfigField for EyebrowsAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.eyebrows = self;
    }
}
impl ConfigField for EyelashesAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.eyelashes = self;
    }
}
impl ConfigField for TeethAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.teeth = self;
    }
}
impl ConfigField for TongueAsset {
    fn apply(self, config: &mut HumanConfig) {
        config.tongue = self;
    }
}
impl ConfigField for Option<HairAsset> {
    fn apply(self, config: &mut HumanConfig) {
        config.hair = self;
    }
}

//
// Morphs List UI
//

/// Marker for morphs list container
#[derive(Component)]
struct MorphsList;

/// Marker for individual morph item with index
#[derive(Component)]
struct MorphItem(usize);

/// Marker for morph slider with index
#[derive(Component)]
struct MorphSlider(usize);

/// Marker for add morph dropdown
#[derive(Component)]
struct AddMorphDropdown;

#[derive(Component)]
struct AddMorphPopup;

#[derive(Component)]
struct MorphFilterInput;

#[derive(Component)]
struct MorphOptionItem(String);

fn spawn_morphs_list_content(parent: &mut ChildSpawnerCommands, morphs: &[Morph], target: Entity) {
    parent
        .spawn((
            MorphsList,
            TargetHuman(target),
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4.0),
                ..default()
            },
        ))
        .with_children(|list| {
            // Current morphs with sliders and remove buttons
            for (idx, morph) in morphs.iter().enumerate() {
                let (min, max) = morph.target.value_range();
                list.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(2.0),
                        ..default()
                    },
                    MorphItem(idx),
                    children![
                        (
                            Node {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            children![
                                    (
                                        Text::new(format!("{:?}", morph.target)),
                                        ThemedText,
                                        TextFont::from_font_size(9.0),
                                        Node {
                                            flex_grow: 1.0,
                                            max_width: px(140.0),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                    ),
                                    (
                                        button(
                                            ButtonProps::default(),
                                            (),
                                            Spawn((
                                                Text::new("X"),
                                                ThemedText,
                                                TextFont::from_font_size(10.0)
                                            )),
                                        ),
                                        observe(
                                            move |_: On<Pointer<Click>>,
                                                  mut human_query: Query<&mut HumanConfig>,
                                                  list_query: Query<
                                                &TargetHuman,
                                                With<MorphsList>,
                                            >| {
                                                if let Ok(target) = list_query.single() {
                                                    if let Ok(mut config) =
                                                        human_query.get_mut(target.0)
                                                    {
                                                        if idx < config.morphs.len() {
                                                            config.morphs.remove(idx);
                                                        }
                                                    }
                                                }
                                            }
                                        ),
                                    ),
                                ],
                        ),
                        (
                            MorphSlider(idx),
                            TargetHuman(target),
                            slider(
                                SliderProps {
                                    min,
                                    max,
                                    value: morph.value,
                                    ..default()
                                },
                                (SliderStep(0.05), SliderPrecision(2)),
                            ),
                            observe(on_morph_slider_change),
                        ),
                    ],
                ));
            }

            // Add button with dropdown
            spawn_add_morph_button(list, target);
        });
}

fn spawn_add_morph_button(parent: &mut ChildSpawnerCommands, target: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(4.0),
            ..default()
        },
        AddMorphDropdown,
        TargetHuman(target),
        children![(
            button(
                ButtonProps::default(),
                (),
                Spawn((
                    Text::new("+ Add Morph"),
                    ThemedText,
                    TextFont::from_font_size(11.0)
                )),
            ),
            observe(toggle_add_morph_dropdown),
        ),],
    ));
}

fn toggle_add_morph_dropdown(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    dropdown_query: Query<(Entity, &TargetHuman), With<AddMorphDropdown>>,
    popup_query: Query<Entity, With<AddMorphPopup>>,
) {
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    let items: Vec<_> = MorphTarget::iter().collect();
    let human_entity = target.0;

    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                AddMorphPopup,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    width: px(280.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                // Filter input
                popup.spawn((
                    Node {
                        padding: UiRect::all(px(4.0)),
                        ..default()
                    },
                    children![(
                        MorphFilterInput,
                        text_input(
                            TextInputProps {
                                placeholder: "Filter morphs...".into(),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ()
                        ),
                    )],
                ));

                // Scrollable list
                popup.spawn(scroll(
                    ScrollProps {
                        width: Val::Percent(100.0),
                        height: px(300.0),
                        overflow: Overflow {
                            x: OverflowAxis::Visible,
                            y: OverflowAxis::Scroll,
                        },
                        flex_direction: FlexDirection::Column,
                        corners: Default::default(),
                        bg_token: tokens::WINDOW_BG,
                        align_items: AlignItems::Stretch,
                    },
                    (),
                    Children::spawn(SpawnIter(items.into_iter().map(move |variant| {
                        let label = format!("{:?}", variant);
                        let filter_name = label.to_lowercase();
                        (
                        MorphOptionItem(filter_name),
                        Node {
                            padding: UiRect::all(px(4.0)),
                            ..default()
                        },
                        bevy::prelude::Button,
                        ThemeBackgroundColor(tokens::BUTTON_BG),
                        observe(move |_: On<Pointer<Click>>,
                                      mut commands: Commands,
                                      popup_query: Query<Entity, With<AddMorphPopup>>,
                                      mut human_query: Query<&mut HumanConfig>| {
                            if let Ok(mut config) = human_query.get_mut(human_entity) {
                                config.morphs.push(Morph::new(variant, 0.0));
                            }
                            if let Ok(popup) = popup_query.single() {
                                commands.entity(popup).despawn();
                            }
                        }),
                        children![(
                            Text::new(label),
                            ThemedText,
                            TextFont::from_font_size(10.0),
                        )],
                    )
                    }))),
                ));
            });
    });
}

fn filter_morph_options(
    filter_query: Query<&TextInputBuffer, (With<MorphFilterInput>, Changed<TextInputBuffer>)>,
    mut items_query: Query<(&MorphOptionItem, &mut Node)>,
) {
    let Ok(buffer) = filter_query.single() else {
        return;
    };
    let filter = buffer.get_text().to_lowercase();

    for (item, mut node) in items_query.iter_mut() {
        node.display = if filter.is_empty() || item.0.contains(&filter) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn on_morph_slider_change(
    trigger: On<ValueChange<f32>>,
    slider_query: Query<(&MorphSlider, &TargetHuman)>,
    mut human_query: Query<&mut HumanConfig>,
    mut commands: Commands,
) {
    let Ok((morph_slider, target)) = slider_query.get(trigger.source) else {
        return;
    };
    let Ok(mut config) = human_query.get_mut(target.0) else {
        return;
    };

    if morph_slider.0 < config.morphs.len() {
        config.morphs[morph_slider.0].value = trigger.value;
    }

    commands
        .entity(trigger.source)
        .insert(SliderValue(trigger.value));
}

//
// Pose UI
//

/// Marker for pose dropdown container
#[derive(Component)]
struct PoseDropdown;

#[derive(Component)]
struct PosePopup;

#[derive(Component)]
struct PoseFilterInput;

#[derive(Component)]
struct PoseOptionItem(String);

fn spawn_pose_list_content(parent: &mut ChildSpawnerCommands, target: Entity) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4.0),
            ..default()
        },
        children![(
            PoseDropdown,
            TargetHuman(target),
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(4.0),
                ..default()
            },
            children![(
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((
                        Text::new("Select Pose..."),
                        ThemedText,
                        TextFont::from_font_size(11.0)
                    )),
                ),
                observe(toggle_pose_dropdown),
            )],
        )],
    ));
}

fn toggle_pose_dropdown(
    _trigger: On<Pointer<Click>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dropdown_query: Query<(Entity, &TargetHuman), With<PoseDropdown>>,
    popup_query: Query<Entity, With<PosePopup>>,
) {
    let Ok((dropdown_entity, target)) = dropdown_query.single() else {
        return;
    };

    if let Ok(popup) = popup_query.single() {
        commands.entity(popup).despawn();
        return;
    }

    // Collect items with optional thumbnails
    let items: Vec<_> = PoseAsset::iter()
        .map(|v| {
            let thumb = v.thumbnail_path().map(|p| asset_server.load::<Image>(p));
            (v, thumb)
        })
        .collect();
    info!("Pose dropdown opened with {} poses", items.len());
    let human_entity = target.0;

    commands.entity(dropdown_entity).with_children(|parent| {
        parent
            .spawn((
                PosePopup,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0),
                    left: px(0.0),
                    width: px(280.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ThemeBackgroundColor(tokens::WINDOW_BG),
                BorderRadius::all(px(4.0)),
                GlobalZIndex(1000),
            ))
            .with_children(|popup| {
                // Filter input
                popup.spawn((
                    Node {
                        padding: UiRect::all(px(4.0)),
                        ..default()
                    },
                    children![(
                        PoseFilterInput,
                        text_input(
                            TextInputProps {
                                placeholder: "Filter poses...".into(),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ()
                        ),
                    )],
                ));

                // Scrollable list
                popup.spawn(scroll(
                    ScrollProps {
                        width: Val::Percent(100.0),
                        height: px(300.0),
                        overflow: Overflow {
                            x: OverflowAxis::Visible,
                            y: OverflowAxis::Scroll,
                        },
                        flex_direction: FlexDirection::Column,
                        corners: Default::default(),
                        bg_token: tokens::WINDOW_BG,
                        align_items: AlignItems::Stretch,
                    },
                    (),
                    Children::spawn(SpawnIter(items.into_iter().map(move |(variant, thumb)| {
                        let label = format!("{}", variant);
                        let filter_name = label.to_lowercase();
                        (
                        PoseOptionItem(filter_name),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: px(6.0),
                            padding: UiRect::all(px(4.0)),
                            ..default()
                        },
                        bevy::prelude::Button,
                        ThemeBackgroundColor(tokens::BUTTON_BG),
                        observe(move |_: On<Pointer<Click>>,
                                      mut commands: Commands,
                                      popup_query: Query<Entity, With<PosePopup>>| {
                            info!("Selected pose: {:?}", variant);
                            commands.entity(human_entity).insert(ApplyPose(variant));

                            if let Ok(popup) = popup_query.single() {
                                commands.entity(popup).despawn();
                            }
                        }),
                        Children::spawn(SpawnWith(move |spawner: &mut ChildSpawner| {
                            if let Some(handle) = thumb.clone() {
                                spawner.spawn((
                                    ImageNode::new(handle),
                                    Node {
                                        width: px(24.0),
                                        height: px(24.0),
                                        ..default()
                                    },
                                ));
                            }
                            spawner.spawn((
                                Text::new(label.clone()),
                                ThemedText,
                                TextFont::from_font_size(10.0),
                            ));
                        })),
                    )
                    }))),
                ));
            });
    });
}

fn filter_pose_options(
    filter_query: Query<&TextInputBuffer, (With<PoseFilterInput>, Changed<TextInputBuffer>)>,
    mut items_query: Query<(&PoseOptionItem, &mut Node)>,
) {
    let Ok(buffer) = filter_query.single() else {
        return;
    };
    let filter = buffer.get_text().to_lowercase();

    for (item, mut node) in items_query.iter_mut() {
        node.display = if filter.is_empty() || item.0.contains(&filter) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Component to trigger pose loading - stores the pose asset enum
#[derive(Component)]
struct ApplyPose(PoseAsset);

/// Component for pose being loaded - stores handle
#[derive(Component)]
struct LoadingPose(Handle<Pose>);

/// System to load pose when ApplyPose is added
fn load_pose_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<(Entity, &ApplyPose), Without<LoadingPose>>,
) {
    for (entity, apply_pose) in query.iter() {
        let bvh_path = apply_pose.0.bvh_path();
        info!("Loading pose: {}", bvh_path);
        let handle: Handle<Pose> = asset_server.load(bvh_path);
        commands.entity(entity).insert(LoadingPose(handle));
    }
}

/// System to apply poses when loaded
fn apply_pose_system(
    mut commands: Commands,
    pose_query: Query<(Entity, &LoadingPose)>,
    pose_assets: Res<Assets<Pose>>,
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mut transform_query: Query<&mut Transform>,
) {
    for (entity, loading_pose) in pose_query.iter() {
        let Some(pose) = pose_assets.get(&loading_pose.0) else {
            continue; // Pose not loaded yet
        };

        // Find bone entities and apply pose
        for child in children_query.iter_descendants(entity) {
            let Ok(name) = name_query.get(child) else {
                continue;
            };
            let bone_name = name.as_str();

            // Check for rotation in pose
            if let Some(pose_rotation) = pose.rotation(bone_name) {
                if let Ok(mut transform) = transform_query.get_mut(child) {
                    // BVH rotation is a delta from bind pose
                    transform.rotation = transform.rotation * pose_rotation;
                }
            }
        }

        // Remove components after applying
        commands
            .entity(entity)
            .remove::<ApplyPose>()
            .remove::<LoadingPose>();
        info!(
            "Applied pose with {} bone rotations",
            pose.bone_rotations.len()
        );
    }
}




#[derive(AssetCollection, Resource)]
struct GltfAnimationAssets {
    #[asset(path = "animations/gltf/main_skeleton.glb")]
    pub main_skeleton: Handle<Gltf>,
    #[asset(path = "animations/gltf/Breathing Idle.glb")]
    pub breathing_idle: Handle<Gltf>,
    /// MH-compatible skeleton from Humentity - bind poses match MH mesh
    #[asset(path = "animations/gltf/mixamo_skeleton.glb")]
    pub mixamo_skeleton: Handle<Gltf>,
}

/// Build GLTF node hierarchy paths recursively
fn build_gltf_paths(
    node_handle: &Handle<bevy::gltf::GltfNode>,
    current_path: &[Name],
    gltf_nodes: &Assets<bevy::gltf::GltfNode>,
    paths: &mut HashMap<String, Vec<Name>>,
) {
    let Some(node) = gltf_nodes.get(node_handle) else {
        return;
    };

    let mut path = current_path.to_vec();
    path.push(Name::new(node.name.clone()));

    // Store path for this node
    paths.insert(node.name.clone(), path.clone());

    // Recurse into children
    for child_handle in &node.children {
        build_gltf_paths(child_handle, &path, gltf_nodes, paths);
    }
}


#[allow(dead_code)]
fn apply_gltf_animation(
    trigger: On<CharacterComplete>,
    mut human_query: Query<(&HumanConfig, &mut Skeleton)>,
    mut commands: Commands,
    assets: Res<GltfAnimationAssets>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<bevy::gltf::GltfNode>>,
    
    children_query: Query<&Children>,
    name_query: Query<&Name>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {    
    // Get config and skeleton
    let (config, skeleton)  = human_query.get_mut(trigger.entity).unwrap();

    // Get the GLTF asset
    let gltf = gltfs.get(&assets.breathing_idle).unwrap();
    if gltf.animations.is_empty() {
        warn!("No animations in GLTF");
        return;
    }

    // Build GLTF hierarchy paths from scene roots
    let mut gltf_paths: HashMap<String, Vec<Name>> = HashMap::default();
    for node_handle in &gltf.nodes {
        if let Some(node) = gltf_nodes.get(node_handle) {
            if !gltf_paths.contains_key(&node.name) {
                build_gltf_paths(node_handle, &[], &gltf_nodes, &mut gltf_paths);
            }
        }
    }

    // Find rig entity with AnimationPlayer
    let mut rig_entity = None;
    for child in children_query.iter_descendants(trigger.entity) {
        if animation_player_query.get(child).is_ok() {
            rig_entity = Some(child);
            break;
        }
    }

    let Some(rig) = rig_entity else {
        warn!("No rig entity with AnimationPlayer found");
        return;
    };

    // Build source->target AnimationTargetId mapping
    // With Humentity's base rotation formula, animations should work directly without runtime corrections
    let mut id_map: HashMap<AnimationTargetId, AnimationTargetId> = HashMap::default();

    // For Mixamo-compatible rigs, build direct mapping
    let needs_retargeting = !matches!(config.rig, RigAsset::Mixamo | RigAsset::MixamoUnity);

    for (source_name, _node_handle) in &gltf.named_nodes {
        // Special case: Armature is the rig root, not a bone
        if source_name.as_ref() == "Armature" {
            let source_id = AnimationTargetId::from_name(&Name::new("Armature"));
            let target_id = AnimationTargetId::from_name(&Name::new("Armature"));
            id_map.insert(source_id, target_id);
            continue;
        }

        // Get target bone name (retarget if needed, otherwise use source name)
        // todo: retarget, but for now going mixamo to mixamo
        let target_name = if skeleton.bone_index(source_name).is_some() {
            Some(source_name.as_ref())
        } else {
            None
        };

        if let Some(target_name) = target_name {
            if let Some(bone_idx) = skeleton.bone_index(target_name) {
                // Build path from bone to root for MH skeleton target
                let mut target_path = vec![Name::new(target_name.to_string())];
                let mut current_idx = bone_idx;
                while let Some(parent_idx) = skeleton.hierarchy[current_idx] {
                    target_path.push(Name::new(skeleton.bones[parent_idx].name.clone()));
                    current_idx = parent_idx;
                }
                target_path.push(Name::new("Armature"));

                let target_id = AnimationTargetId::from_names(target_path.iter().rev());

                // Source ID uses FULL path from GLTF hierarchy
                if let Some(source_path) = gltf_paths.get(source_name.as_ref()) {
                    let source_id = AnimationTargetId::from_names(source_path.iter());
                    id_map.insert(source_id, target_id);
                }
            }
        }
    }

    info!(
        "Built retarget map with {} bones (retargeting={})",
        id_map.len(),
        needs_retargeting
    );

    // Debug: print Hips mapping specifically
    // if let Some(hips_path) = gltf_paths.get("mixamorig:Hips") {
    //     let source_id = AnimationTargetId::from_names(hips_path.iter());
    //     info!("Hips source path: {:?}, source_id exists in map: {}",
    //         hips_path.iter().map(|n| n.as_str()).collect::<Vec<_>>(),
    //         id_map.contains_key(&source_id));
    // }

    // Debug: print first few mappings
    // for (_i, (source_name, _)) in gltf.named_nodes.iter().take(3).enumerate() {
    //     if let Some(path) = gltf_paths.get(source_name.as_ref()) {
    //         let path_str: Vec<_> = path.iter().map(|n| n.as_str()).collect();
    //         info!("GLTF path for '{}': {:?}", source_name, path_str);
    //     }
    // }

    // Build animation graph with retargeted clips
    let mut graph = AnimationGraph::new();
    let mut nodes = Vec::new();

    for clip_handle in &gltf.animations {
        let Some(source_clip) = animation_clips.get(clip_handle) else {
            continue;
        };

        // Create retargeted clip - copy translation/rotation curves, skip scale
        // Mixamo animations use 0.01 scale (cm to m) which breaks our character
        let mut new_clip = AnimationClip::default();
        let mut curves_copied = 0;
        let mut scale_skipped = 0;
        let mut unmapped = 0;

        // Transform::scale is field index 2 (after translation=0, rotation=1)
        let transform_type_id = TypeId::of::<Transform>();
        const SCALE_FIELD_INDEX: usize = 2;

        for (source_id, curves) in source_clip.curves().iter() {
            if let Some(&target_id) = id_map.get(source_id) {
                for curve in curves.iter() {
                    // Check if this is a scale curve - skip it
                    let is_scale = match curve.0.evaluator_id() {
                        EvaluatorId::ComponentField(hashed) => {
                            let (type_id, field_idx) = **hashed;
                            type_id == transform_type_id && field_idx == SCALE_FIELD_INDEX
                        }
                        _ => false,
                    };

                    if is_scale {
                        scale_skipped += 1;
                    } else {
                        new_clip.add_variable_curve_to_target(target_id, curve.clone());
                    }
                }
                curves_copied += 1;
            } else {
                unmapped += 1;
            }
        }

        info!("Retargeted {} targets, {} unmapped, {} scale curves skipped", curves_copied, unmapped, scale_skipped);

        if curves_copied > 0 {
            let new_handle = animation_clips.add(new_clip);
            let node = graph.add_clip(new_handle, 1.0, graph.root);
            nodes.push(node);
        }
    }

    info!("Created {} animation nodes", nodes.len());

    let graph_handle = animation_graphs.add(graph);

    // Attach graph to rig
    commands
        .entity(rig)
        .insert(AnimationGraphHandle(graph_handle));

    // Play first animation
    if let Some(&first_node) = nodes.first() {
        if let Ok(mut player) = animation_player_query.get_mut(rig) {
            player.play(first_node).repeat();
            info!("Started retargeted GLTF animation");
        }
    }
}


// #[derive(AssetCollection, Resource)]
// struct DipAssets {
//     #[asset(path = "animations/dip/dip_throw_ball_motion_only.npy")]
//     pub throw_ball: Handle<MotionData>,
// }

// #[allow(dead_code)]
// fn apply_dip_animation(
//     trigger: On<CharacterComplete>,
//     mut commands: Commands,
//     dip_assets: Res<DipAssets>,
//     query: Query<(Entity, &Skeleton)>,
//     children_query: Query<&Children>,
//     mut animation_player_query: Query<&mut AnimationPlayer>,
//     motion_assets: Res<Assets<MotionData>>,
//     mut animation_clips: ResMut<Assets<AnimationClip>>,
//     mut animation_graphs: ResMut<Assets<AnimationGraph>>,
// ) {
//     let (character, skeleton) = query.get(trigger.entity).unwrap();
//     // Wait for motion data to load
//     let Some(motion_data) = motion_assets.get(&dip_assets.throw_ball) else {
//         warn!("DIP motion data not loaded yet");
//         return;
//     };

//     info!(
//         "Applying MDM motion: {} frames, {:.1}s",
//         motion_data.frame_count(),
//         motion_data.duration
//     );

//     // Find rig entity with AnimationPlayer
//     let mut rig_entity = None;
//     for child in children_query.iter_descendants(character) {
//         if animation_player_query.get(child).is_ok() {
//             rig_entity = Some(child);
//             break;
//         }
//     }

//     let Some(rig) = rig_entity else {
//         warn!("No rig entity with AnimationPlayer found");
//         return;
//     };

//     // Convert motion data to animation clip
//     let clip = motion_data.to_animation_clip(skeleton);
//     let clip_handle = animation_clips.add(clip);

//     // Create animation graph
//     let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
//     let graph_handle = animation_graphs.add(graph);

//     // Attach graph to rig and play
//     commands
//         .entity(rig)
//         .insert(AnimationGraphHandle(graph_handle.clone()));

//     // Also mark character so we don't re-apply
//     commands
//         .entity(character)
//         .insert(AnimationGraphHandle(graph_handle));

//     if let Ok(mut player) = animation_player_query.get_mut(rig) {
//         player.play(node_index).repeat();
//     }
// }
