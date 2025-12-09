#[path = "common/mod.rs"]
mod common;
pub use common::*;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    feathers::{FeathersPlugin, controls::*, dark_theme::create_dark_theme, theme::*, tokens},
    picking::mesh_picking::MeshPickingPlugin,
    prelude::*,
    ui::Checked,
    ui_widgets::*,
};
use bevy_make_human::{prelude::*, ui::text_input::handle_text_input_focus};
use bevy_ui_text_input::TextInputPlugin;
/// Marker for the config panel
#[derive(Component)]
struct ConfigPanel;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        FeathersPlugin,
        MeshPickingPlugin,
        PhysicsPlugins::default(),
        MakeHumanPlugin::default(),
        TextInputPlugin,
        CommonPlugin, // camera and egui editor
    ))
    .insert_resource(UiTheme(create_dark_theme()));

    // Register filter dropdowns for component enum types
    register_filter_dropdown::<Hair>(&mut app);
    register_filter_dropdown::<Eyes>(&mut app);
    register_filter_dropdown::<Eyebrows>(&mut app);
    register_filter_dropdown::<Eyelashes>(&mut app);
    register_filter_dropdown::<Teeth>(&mut app);
    register_filter_dropdown::<Tongue>(&mut app);
    register_filter_dropdown::<Rig>(&mut app);

    app
        // .init_collection::<DipAssets>() // testing dip generated animations
        // .init_collection::<MixamoAssets>() // mixamo fbx animations
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, 
            // stop camera movement when typing in text inputs
            handle_text_input_focus::<CameraFree>.run_if(resource_changed::<bevy::input_focus::InputFocus>),        
        )
        .run()
    // .add_systems(
    //     Update,
    //     (update_play_button_text)
    //         .run_if(in_state(GameState::Ready)),
    // )
    // .add_systems(
    //     Update,
    //     (
    //         // paint_joint_labels,
    //         // setup_gltf_animations.run_if(in_state(GameState::Ready)),
    //         // draw_joint_axes,
    //         // update_config_panel,
    //         // filter_clothing_options,
    //         // filter_morph_options,
    //         // filter_pose_options,
    //         // load_pose_system,
    //         // apply_pose_system,
    //         // filter_dropdown_options::<SkinAsset>,
    //         // filter_dropdown_options::<RigAsset>,
    //         // filter_dropdown_options::<Option<HairAsset>>,
    //         // filter_dropdown_options::<ProxyMesh>,
    //         // filter_dropdown_options::<EyesAsset>,
    //         // filter_dropdown_options::<EyeMaterialAsset>,
    //         // filter_dropdown_options::<EyebrowsAsset>,
    //         // filter_dropdown_options::<EyelashesAsset>,
    //         // filter_dropdown_options::<TeethAsset>,
    //         // filter_dropdown_options::<TongueAsset>,
    //     ),
    // )
    // testing different animations systems
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        CameraFree::default(),
        Transform::from_xyz(0.0, 3., -5.0).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y),
    ));

    // Lighting
    commands.spawn((
        DirectionalLight::default(),
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
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    commands
        .spawn((
            Name::new("Bob"),
            Human,
            Rig::Mixamo,
            Skin {
                mesh: SkinMesh::MaleGeneric,
                material: SkinMaterial::YoungCaucasianMale,
            },
            Eyes::LowPolyBluegreen,
            Hair::CulturalibreHair02,
            Eyebrows::Eyebrow006,
            Eyelashes::Eyelashes01,
            Teeth::TeethBase,
            Tongue::Tongue01,
            Clothing(vec![
                ClothingAsset::ToigoMaleSuit3,
                ClothingAsset::ToigoAnkleBootsMale,
            ]),
            Phenotype {
                race: Race::Caucasian,
                gender: 1.0,
                age: 0.5,
                muscle: 0.3,
                weight: 0.4,
                ..default()
            },
            Transform::from_xyz(-1.0, 0.0, 0.0),
        ))
        .observe(on_human_click);

    commands
        .spawn((
            Name::new("Sarah"),
            Human,
            Rig::Mixamo,
            Skin {
                mesh: SkinMesh::FemaleGeneric,
                material: SkinMaterial::YoungCaucasianFemale,
            },
            Eyes::LowPolyBluegreen,
            Hair::ElvsLaraHair,
            Eyebrows::Eyebrow006,
            Eyelashes::Eyelashes04,
            Teeth::TeethBase,
            Tongue::Tongue01,
            Clothing(vec![
                ClothingAsset::ElvsGoddessDress8,
                //ClothingAsset::ToigoAnkleBootsMale,
            ]),
            Phenotype {
                race: Race::Caucasian,
                gender: 0.0,
                age: 0.5,
                muscle: 0.3,
                weight: 0.4,
                ..default()
            },
            Transform::from_xyz(1.0, 0.0, 0.0),
        ))
        .observe(on_human_click);

    // Human is spawned in setup_human after GLTF assets are loaded
    // so we can extract reference skeleton rotations
}

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
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((
                        Text::new("Add Human"),
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
        Visibility::Visible,
        children![scroll(
            ScrollProps::vertical(percent(50.)),
            (),
            children![(
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8.0),
                    overflow: Overflow::hidden(),
                    ..default()
                },
                ConfigPanel,
            ),],
        )],
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
            (
                checkbox((), Spawn((Text::new("Skeleton"), ThemedText))),
                observe(toggle_skeleton),
            ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Skin"), ThemedText))),
            //     observe(handle_check_visibility::<SkinMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Hair"), ThemedText))),
            //     observe(handle_check_visibility::<HairMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Eyes"), ThemedText))),
            //     observe(handle_check_visibility::<EyesMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Teeth"), ThemedText))),
            //     observe(handle_check_visibility::<TeethMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Tongue"), ThemedText))),
            //     observe(handle_check_visibility::<TongueMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Eyebrows"), ThemedText))),
            //     observe(handle_check_visibility::<EyebrowsMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Eyelashes"), ThemedText))),
            //     observe(handle_check_visibility::<EyelashesMesh>),
            // ),
            // (
            //     checkbox(Checked, Spawn((Text::new("Clothes"), ThemedText))),
            //     observe(handle_check_visibility::<ClothesMesh>),
            // ),
        ],
    ));
}

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
    config_panel: Query<Entity, With<ConfigPanel>>,
    human_query: Query<HumanQuery>,
) {
    info!("Human clicked: {:?}", trigger.entity);
    let Ok(panel_entity) = config_panel.single() else {
        return;
    };

    commands
        .entity(panel_entity)
        .despawn_children()
        .insert(Visibility::Visible);

    let h = human_query.get(trigger.entity).unwrap();
    let human_entity = h.entity;

    commands.entity(panel_entity).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new(format!(
                "Configure: {}",
                h.name
                    .map_or_else(|| "Unnamed".to_string(), |n| n.to_string())
            )),
            ThemedText,
            TextFont::from_font_size(14.0),
        ));

        // Rig
        parent.spawn((Text::new("Rig"), ThemedText));
        parent.spawn(dropdown_filter::<Rig>(human_entity, Some(*h.rig)));

        // Hair
        parent.spawn((Text::new("Hair"), ThemedText));
        parent.spawn(dropdown_filter::<Hair>(human_entity, Some(*h.hair)));

        // Eyes
        parent.spawn((Text::new("Eyes"), ThemedText));
        parent.spawn(dropdown_filter::<Eyes>(human_entity, Some(*h.eyes)));

        // Eyebrows
        parent.spawn((Text::new("Eyebrows"), ThemedText));
        parent.spawn(dropdown_filter::<Eyebrows>(human_entity, Some(*h.eyebrows)));

        // Eyelashes
        parent.spawn((
            Text::new("Eyelashes"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            ThemedText,
        ));
        parent.spawn(dropdown_filter::<Eyelashes>(
            human_entity,
            Some(*h.eyelashes),
        ));

        // Teeth
        parent.spawn((Text::new("Teeth"), ThemedText));
        parent.spawn(dropdown_filter::<Teeth>(human_entity, Some(*h.teeth)));

        // Tongue
        parent.spawn((Text::new("Tongue"), ThemedText));
        parent.spawn(dropdown_filter::<Tongue>(human_entity, Some(*h.tongue)));
    });
}

// Body section

// spawn_dropdown::<SkinMesh>("Mesh", human.skin.mesh, human.entity);
// spawn_dropdown::<Rig>("Rig", human.rig, human.entity);
// spawn_dropdown::<SkinMaterial>(
//     "Material",
//     human.skin.material,
//     human.entity,
// );
//spawn_floor_offset_slider(human.floor_offset.0, human.entity);

/* // Face section
// spawn_collapsible_section(
//     scroll_content,
//     "Face",
//     face_collapsed,
//     |content| {
//         spawn_optional_dropdown::<HairAsset>(
//             content,
//             "Hair",
//             hair,
//             human_entity,
//         );
//         spawn_dropdown::<EyesAsset>(
//             content,
//             "Eyes",
//             eyes,
//             human_entity,
//         );
//         spawn_dropdown::<EyeMaterialAsset>(
//             content,
//             "Eye Color",
//             eye_material,
//             human_entity,
//         );
//         spawn_dropdown::<EyebrowsAsset>(
//             content,
//             "Eyebrows",
//             eyebrows,
//             human_entity,
//         );
//         spawn_dropdown::<EyelashesAsset>(
//             content,
//             "Eyelashes",
//             eyelashes,
//             human_entity,
//         );
//     },
// );

// // Mouth section
// spawn_collapsible_section(
//     scroll_content,
//     "Mouth",
//     mouth_collapsed,
//     |content| {
//         spawn_dropdown::<TeethAsset>(
//             content,
//             "Teeth",
//             teeth,
//             human_entity,
//         );
//         spawn_dropdown::<TongueAsset>(
//             content,
//             "Tongue",
//             tongue,
//             human_entity,
//         );
//     },
// );

// // Clothing section
// spawn_collapsible_section(
//     scroll_content,
//     "Clothing",
//     clothing_collapsed,
//     |content| {
//         spawn_clothing_list_content(
//             content,
//             &clothing,
//             clothing_offset,
//             human_entity,
//         );
//     },
// );

// // Phenotype section
// spawn_collapsible_section(
//     scroll_content,
//     "Phenotype",
//     phenotype_collapsed,
//     |content| {
//         spawn_phenotype_sliders_content(
//             content,
//             &phenotype,
//             human_entity,
//         );
//     },
// );

// // Morphs section
// spawn_collapsible_section(
//     scroll_content,
//     "Morphs",
//     morphs_collapsed,
//     |content| {
//         spawn_morphs_list_content(content, &morphs, human_entity);
//     },
// );

// // Pose section
// spawn_collapsible_section(
//     scroll_content,
//     "Pose",
//     pose_collapsed,
//     |content| {
//         spawn_pose_list_content(content, human_entity);
//     },
// );
*/
// /// System to apply poses when loaded
// fn apply_pose_system(
//     mut commands: Commands,
//     pose_query: Query<(Entity, &LoadingPose)>,
//     pose_assets: Res<Assets<Pose>>,
//     children_query: Query<&Children>,
//     name_query: Query<&Name>,
//     mut transform_query: Query<&mut Transform>,
// ) {
//     for (entity, loading_pose) in pose_query.iter() {
//         let Some(pose) = pose_assets.get(&loading_pose.0) else {
//             continue; // Pose not loaded yet
//         };

//         // Find bone entities and apply pose
//         for child in children_query.iter_descendants(entity) {
//             let Ok(name) = name_query.get(child) else {
//                 continue;
//             };
//             let bone_name = name.as_str();

//             // Check for rotation in pose
//             if let Some(pose_rotation) = pose.rotation(bone_name) {
//                 if let Ok(mut transform) = transform_query.get_mut(child) {
//                     // BVH rotation is a delta from bind pose
//                     transform.rotation = transform.rotation * pose_rotation;
//                 }
//             }
//         }

//         // Remove components after applying
//         commands
//             .entity(entity)
//             .remove::<ApplyPose>()
//             .remove::<LoadingPose>();
//         info!(
//             "Applied pose with {} bone rotations",
//             pose.bone_rotations.len()
//         );
//     }
// }
