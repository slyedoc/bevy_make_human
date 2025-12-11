#[path = "common/mod.rs"]
mod common;
pub use common::*;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    feathers::{
        FeathersPlugins, controls::*, dark_theme::create_dark_theme,
        rounded_corners::RoundedCorners, theme::*, tokens,
    },
    picking::{hover::Hovered, mesh_picking::MeshPickingPlugin},
    prelude::*,
    ui::Checked,
    ui_widgets::*,
};
use bevy_make_human::{prelude::*, ui::text_input::handle_text_input_focus};
use bevy_ui_text_input::{TextInputContents, TextInputPlugin};
use strum::IntoEnumIterator;
/// Marker for the config panel
#[derive(Component)]
struct ConfigPanel;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        FeathersPlugins,
        MeshPickingPlugin,
        PhysicsPlugins::default(),
        MakeHumanPlugin::default(),
        TextInputPlugin,
        CommonPlugin, // camera and egui editor
    ))
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, (setup, setup_ui))
    .add_systems(
        Update,
        // stop camera movement when typing in text inputs
        (
            handle_text_input_focus::<CameraFree>
                .run_if(resource_changed::<bevy::input_focus::InputFocus>),
            filter_text_changed::<DropdownFilterInput, Dropdown>,
            filter_text_changed::<ClothingFilterInput, ClothingSection>,
            filter_text_changed::<MorphFilterInput, MorphsSection>,
        ),
    )
    .run()
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
            SkinMesh::MaleGeneric,
            SkinMaterial::YoungCaucasianMale,
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
            SkinMesh::FemaleGeneric,
            SkinMaterial::YoungCaucasianFemale,
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
            width: px(400.0),
            top: px(20.0),
            left: px(20.0),
            bottom: px(20.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(2.0)),
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        BorderRadius::all(px(8.0)),
        ConfigPanel,
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
    config_panel: Single<Entity, With<ConfigPanel>>,
    human_query: Query<HumanQuery>,
) {
    commands
        .entity(*config_panel)
        .despawn_children()
        .insert(Visibility::Visible);

    let e = trigger.entity;
    let h = human_query.get(trigger.entity).unwrap();
    commands.entity(*config_panel).with_child(scroll(
        ScrollProps::vertical(percent(100.)),
        (),
        children![
            (
                Name::new("HeaderRow"),
                Node {
                    flex_direction: FlexDirection::Row,
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![
                    (
                        Text::new(format!(
                            "Name: {}",
                            h.name
                                .map_or_else(|| "Unnamed".to_string(), |n| n.to_string())
                        )),
                        ThemedText,
                        TextFont::from_font_size(14.0),
                    ),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                        children![
                            (
                                button(
                                    ButtonProps::default(),
                                    (),
                                    Spawn((Text::new("-"), ThemedText)),
                                ),
                                observe(collapse_all),
                            ),
                            (
                                button(
                                    ButtonProps::default(),
                                    (),
                                    Spawn((Text::new("+"), ThemedText)),
                                ),
                                observe(expand_all),
                            ),
                        ],
                    ),
                ],
            ),
            collapsible("General", true, children![
                dropdown_mh::<Rig>(e, *h.rig),
                dropdown_mh_thumb::<SkinMesh>(e, *h.skin_mesh),
                dropdown_mh_thumb::<SkinMaterial>(e, *h.skin_material),
                offset_slider::<FloorOffset>(e, "Floor", h.floor_offset.0, -0.1, 0.1),
            ]),
            collapsible("Head", true, children![
                dropdown_mh_thumb::<Hair>(e, *h.hair),
                dropdown_mh_thumb::<Eyes>(e, *h.eyes),
                dropdown_mh_thumb::<Eyebrows>(e, *h.eyebrows),
                dropdown_mh_thumb::<Eyelashes>(e, *h.eyelashes),
                dropdown_mh_thumb::<Teeth>(e, *h.teeth),
                dropdown_mh_thumb::<Tongue>(e, *h.tongue),
            ]),
            collapsible("Clothes", true, children![
                clothing_section(e, &h.clothing),
                offset_slider::<ClothingOffset>(e, "Clothing", h.clothing_offset.0, 0.0, 0.01),
            ]),
            collapsible("Morphs", false, children![
                morphs_section(e, &h.morphs),
            ]),
        ],
    ));
}

/// Collapsible section with title and expandable content
fn collapsible<C: Bundle>(title: &'static str, expanded: bool, content: C) -> impl Bundle {
    (
        Name::new(format!("Collapsible{}", title)),
        Collapsible,
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            padding: UiRect::top(Val::Px(4.0)),
            ..default()
        },
        children![
            (
                Name::new("CollapsibleHeader"),
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((
                        Node { width: Val::Percent(100.0), ..default() },
                        Text::new(format!("{} {}", if expanded { "v" } else { ">" }, title)),
                        ThemedText,
                        TextFont { font_size: 12.0, ..default() },
                    )),
                ),
                observe(on_collapsible_toggle),
            ),
            (
                Name::new("CollapsibleContent"),
                CollapsibleContent,
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    padding: UiRect::left(Val::Px(8.0)),
                    display: if expanded { Display::Flex } else { Display::None },
                    ..default()
                },
                content,
            ),
        ],
    )
}

/// Marker for collapsible section
#[derive(Component)]
struct Collapsible;

/// Marker for collapsible content
#[derive(Component)]
struct CollapsibleContent;

/// Collapse all collapsible sections
fn collapse_all(
    _trigger: On<Pointer<Click>>,
    mut content_query: Query<&mut Node, With<CollapsibleContent>>,
    mut text_query: Query<&mut Text>,
    collapsible_query: Query<&Children, With<Collapsible>>,
    children_query: Query<&Children>,
) {
    for mut node in content_query.iter_mut() {
        node.display = Display::None;
    }
    // Update all header arrows
    for children in collapsible_query.iter() {
        for child in children.iter() {
            for desc in children_query.iter_descendants(child) {
                if let Ok(mut text) = text_query.get_mut(desc) {
                    if text.0.starts_with("v") {
                        text.0 = text.0.replacen("v", ">", 1);
                    }
                    break;
                }
            }
            break;
        }
    }
}

/// Expand all collapsible sections
fn expand_all(
    _trigger: On<Pointer<Click>>,
    mut content_query: Query<&mut Node, With<CollapsibleContent>>,
    mut text_query: Query<&mut Text>,
    collapsible_query: Query<&Children, With<Collapsible>>,
    children_query: Query<&Children>,
) {
    for mut node in content_query.iter_mut() {
        node.display = Display::Flex;
    }
    // Update all header arrows
    for children in collapsible_query.iter() {
        for child in children.iter() {
            for desc in children_query.iter_descendants(child) {
                if let Ok(mut text) = text_query.get_mut(desc) {
                    if text.0.starts_with(">") {
                        text.0 = text.0.replacen(">", "v", 1);
                    }
                    break;
                }
            }
            break;
        }
    }
}

/// Toggle collapsible on header click
fn on_collapsible_toggle(
    trigger: On<Pointer<Click>>,
    parent_query: Query<&ChildOf>,
    collapsible_query: Query<&Collapsible>,
    children_query: Query<&Children>,
    mut content_query: Query<&mut Node, With<CollapsibleContent>>,
    mut text_query: Query<&mut Text>,
) {
    // Find Collapsible parent
    let Some(collapsible) = parent_query
        .iter_ancestors(trigger.entity)
        .find(|e| collapsible_query.get(*e).is_ok())
    else {
        return;
    };

    // Find content and toggle display
    for child in children_query.iter_descendants(collapsible) {
        if let Ok(mut node) = content_query.get_mut(child) {
            let is_expanded = node.display == Display::Flex;
            node.display = if is_expanded { Display::None } else { Display::Flex };

            // Update header text arrow
            for desc in children_query.iter_descendants(trigger.entity) {
                if let Ok(mut text) = text_query.get_mut(desc) {
                    let current = text.0.clone();
                    if current.starts_with("v") {
                        text.0 = current.replacen("v", ">", 1);
                    } else if current.starts_with(">") {
                        text.0 = current.replacen(">", "v", 1);
                    }
                    break;
                }
            }
            break;
        }
    }
}

/// Slider for offset values
fn offset_slider<T: Component + Default + From<f32>>(
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
                TextFont { font_size: 12.0, ..default() },
                ThemedText
            ),
            (
                slider(
                    SliderProps { value, min, max },
                    (),
                ),
                observe(on_offset_change::<T>(human_entity)),
            ),
        ],
    )
}

/// Handler for offset slider changes
fn on_offset_change<T: Component + Default + From<f32>>(
    human_entity: Entity,
) -> impl FnMut(On<ValueChange<f32>>, Commands) {
    move |trigger: On<ValueChange<f32>>, mut commands: Commands| {
        // Update human component
        commands.entity(human_entity).insert(T::from(trigger.value));
        // Update slider UI
        commands.entity(trigger.source).insert(SliderValue(trigger.value));
    }
}

/// Clothing section with list of current items and add button
fn clothing_section(human_entity: Entity, clothing: &Clothing) -> impl Bundle {
    let items: Vec<_> = clothing
        .iter()
        .enumerate()
        .map(|(idx, item)| clothing_item_row(human_entity, idx, *item))
        .collect();

    (
        Name::new("ClothingSection"),
        ClothingSection,
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        children![
            (
                Text::new("Clothing"),
                TextFont { font_size: 12.0, ..default() },
                ThemedText
            ),
            (
                Name::new("ClothingList"),
                ClothingList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                Children::spawn(SpawnIter(items.into_iter())),
            ),
            (
                Name::new("AddClothingButton"),
                ClothingAddButton,
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new("+ Add Clothing"), ThemedText)),
                ),
                observe(on_open_clothing_menu(human_entity)),
            ),
        ],
        observe(on_clothing_select(human_entity)),
        observe(on_clothing_remove(human_entity)),
        observe(on_clothing_close),
        observe(on_clothing_filter),
    )
}

/// Marker for clothing section
#[derive(Component)]
struct ClothingSection;

/// Marker for clothing list container
#[derive(Component)]
struct ClothingList;

/// Marker for add clothing button
#[derive(Component)]
struct ClothingAddButton;

/// Event for selecting clothing to add
#[derive(EntityEvent)]
struct ClothingSelect {
    entity: Entity,
    item: ClothingAsset,
}

/// Event for removing clothing
#[derive(EntityEvent)]
struct ClothingRemove {
    entity: Entity,
    idx: usize,
}

/// Event for closing clothing menu
#[derive(EntityEvent)]
struct ClothingClose {
    entity: Entity,
}

/// Single clothing item row with name and remove button
fn clothing_item_row(_human_entity: Entity, idx: usize, item: ClothingAsset) -> impl Bundle {
    (
        Name::new(format!("ClothingItem_{}", idx)),
        ClothingItem(idx),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            overflow: Overflow::clip(),
            ..default()
        },
        children![
            (
                Text::new(item.to_string()),
                TextFont { font_size: 12.0, ..default() },
                ThemedText,
                Node { flex_grow: 1.0, ..default() },
            ),
            (
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    ..default()
                },
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("×"), ThemedText)),
                    ),
                    observe(on_clothing_item_remove_click),
                )],
            ),
        ],
    )
}

/// Marker for clothing item with index
#[derive(Component)]
struct ClothingItem(usize);

/// Marker for clothing menu
#[derive(Component)]
struct ClothingMenu;

/// Marker for clothing option in menu
#[derive(Component)]
struct ClothingOption(ClothingAsset);

/// Marker for clothing filter input
#[derive(Component)]
struct ClothingFilterInput;

/// Marker for clothing options container
#[derive(Component)]
struct ClothingOptionsContainer;

/// Click handler for remove button - triggers ClothingRemove event
fn on_clothing_item_remove_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    parent_query: Query<&ChildOf>,
    item_query: Query<&ClothingItem>,
    section_query: Query<&ClothingSection>,
) {
    // Find ClothingItem parent to get index
    for ancestor in parent_query.iter_ancestors(trigger.entity) {
        if let Ok(item) = item_query.get(ancestor) {
            // Find ClothingSection ancestor to trigger event
            for section_ancestor in parent_query.iter_ancestors(ancestor) {
                if section_query.get(section_ancestor).is_ok() {
                    commands.trigger(ClothingRemove {
                        entity: section_ancestor,
                        idx: item.0,
                    });
                    return;
                }
            }
        }
    }
}

/// Handler for ClothingRemove event
fn on_clothing_remove(
    human_entity: Entity,
) -> impl FnMut(On<ClothingRemove>, Commands, Query<&mut Clothing>, Query<&Children>, Query<Entity, With<ClothingList>>) {
    move |trigger: On<ClothingRemove>,
          mut commands: Commands,
          mut clothing_query: Query<&mut Clothing>,
          children_query: Query<&Children>,
          list_query: Query<Entity, With<ClothingList>>| {
        if let Ok(mut clothing) = clothing_query.get_mut(human_entity) {
            if trigger.idx < clothing.len() {
                clothing.remove(trigger.idx);
                commands.entity(human_entity).insert(HumanDirty);

                // Update UI - find ClothingList and rebuild children
                for child in children_query.iter_descendants(trigger.entity) {
                    if let Ok(list_entity) = list_query.get(child) {
                        commands.entity(list_entity).despawn_children();
                        for (idx, item) in clothing.iter().enumerate() {
                            commands.entity(list_entity).with_child(
                                clothing_item_row(human_entity, idx, *item)
                            );
                        }
                        break;
                    }
                }
            }
        }
    }
}

/// Handler for ClothingSelect event
fn on_clothing_select(
    human_entity: Entity,
) -> impl FnMut(On<ClothingSelect>, Commands, Query<&mut Clothing>, Query<&Children>, Query<Entity, With<ClothingList>>) {
    move |trigger: On<ClothingSelect>,
          mut commands: Commands,
          mut clothing_query: Query<&mut Clothing>,
          children_query: Query<&Children>,
          list_query: Query<Entity, With<ClothingList>>| {
        // Close menu
        commands.trigger(ClothingClose {
            entity: trigger.entity,
        });

        if let Ok(mut clothing) = clothing_query.get_mut(human_entity) {
            if !clothing.contains(&trigger.item) {
                clothing.push(trigger.item);
                commands.entity(human_entity).insert(HumanDirty);

                // Update UI - find ClothingList and add new item
                for child in children_query.iter_descendants(trigger.entity) {
                    if let Ok(list_entity) = list_query.get(child) {
                        let idx = clothing.len() - 1;
                        commands.entity(list_entity).with_child(
                            clothing_item_row(human_entity, idx, trigger.item)
                        );
                        break;
                    }
                }
            }
        }
    }
}

/// Handler for ClothingClose event
fn on_clothing_close(
    trigger: On<ClothingClose>,
    mut commands: Commands,
    children_query: Query<&Children>,
    menu_query: Query<&ClothingMenu>,
) {
    for child in children_query.iter_descendants(trigger.entity) {
        if menu_query.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

/// Handler for ClothingFilter event
fn on_clothing_filter(
    trigger: On<FilterOptions>,
    children_query: Query<&Children>,
    options_container: Query<Entity, With<ClothingOptionsContainer>>,
    mut query: Query<(&ClothingOption, &mut Node)>,
) {
    info!("filter clothing options: {}", trigger.filter);
    for child in children_query.iter_descendants(trigger.entity) {
        if let Ok(container) = options_container.get(child) {
            for c in children_query.iter_descendants(container) {
                if let Ok((option, mut node)) = query.get_mut(c) {
                    let label = option.0.to_string().to_lowercase();
                    let filter = trigger.filter.to_lowercase();
                    let show = filter.is_empty() || label.contains(&filter);
                    node.display = if show { Display::Flex } else { Display::None };
                }
            }
            break;
        }
    }
}

/// Handler for opening clothing menu
fn on_open_clothing_menu(
    human_entity: Entity,
) -> impl FnMut(On<Pointer<Click>>, Commands, Res<AssetServer>, Query<&Children>, Query<&ClothingMenu>, Query<&ChildOf>, Query<&ClothingSection>) {
    move |trigger: On<Pointer<Click>>,
          mut commands: Commands,
          asset_server: Res<AssetServer>,
          children_query: Query<&Children>,
          menu_query: Query<&ClothingMenu>,
          parent_query: Query<&ChildOf>,
          section_query: Query<&ClothingSection>| {
        // Find ClothingSection ancestor
        let section_entity = parent_query
            .iter_ancestors(trigger.entity)
            .find(|e| section_query.get(*e).is_ok())
            .unwrap_or(trigger.entity);

        // Check if menu already open
        for child in children_query.iter_descendants(section_entity) {
            if menu_query.get(child).is_ok() {
                return;
            }
        }

        let options: Vec<_> = ClothingAsset::iter()
            .map(|item| {
                let label = item.to_string();
                let image = asset_server.load::<Image>(item.thumb());
                (
                    Name::new(label.clone()),
                    ClothingOption(item),
                    Node {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        min_height: Val::Px(28.0),
                        ..default()
                    },
                    children![
                        (
                            Node {
                                width: Val::Px(24.0),
                                height: Val::Px(24.0),
                                ..default()
                            },
                            ImageNode { image, ..default() },
                        ),
                        (
                            button(
                                ButtonProps::default(),
                                (),
                                Spawn((Text::new(label), ThemedText)),
                            ),
                            observe(on_clothing_option_click(human_entity, item)),
                        ),
                    ],
                )
            })
            .collect();

        commands.entity(section_entity).with_child((
            ClothingMenu,
            Name::new("ClothingMenu"),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            },
            ThemeBackgroundColor(tokens::WINDOW_BG),
            ZIndex(10),
            children![
                (
                    ClothingFilterInput,
                    text_input(
                        TextInputProps {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            placeholder: "Filter...".to_string(),
                            corners: RoundedCorners::Top,
                            ..default()
                        },
                        TextInputContents::default(),
                    ),
                ),
                scroll(
                    ScrollProps::vertical(px(300.0)),
                    (ClothingOptionsContainer, Name::new("ClothingOptions")),
                    Children::spawn(SpawnIter(options.into_iter())),
                ),
            ],
            Hovered(false),
            observe(on_hover_exit),
        ));
    }
}

/// Click handler for clothing option - triggers ClothingSelect event
fn on_clothing_option_click(
    _human_entity: Entity,
    item: ClothingAsset,
) -> impl FnMut(On<Pointer<Click>>, Commands, Query<&ChildOf>, Query<&ClothingSection>) {
    move |trigger: On<Pointer<Click>>,
          mut commands: Commands,
          parent_query: Query<&ChildOf>,
          section_query: Query<&ClothingSection>| {
        // Find ClothingSection ancestor
        if let Some(section) = parent_query
            .iter_ancestors(trigger.entity)
            .find(|e| section_query.get(*e).is_ok())
        {
            commands.trigger(ClothingSelect {
                entity: section,
                item,
            });
        }
    }
}

// ==================== MORPHS SECTION ====================

/// Morphs section with list of morphs and add button
fn morphs_section(human_entity: Entity, morphs: &Morphs) -> impl Bundle {
    let items: Vec<_> = morphs
        .iter()
        .enumerate()
        .map(|(idx, morph)| morph_item_row(human_entity, idx, morph))
        .collect();

    (
        Name::new("MorphsSection"),
        MorphsSection,
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::top(Val::Px(8.0)),
            ..default()
        },
        children![
            (
                Text::new("Morphs"),
                TextFont { font_size: 12.0, ..default() },
                ThemedText
            ),
            (
                Name::new("MorphsList"),
                MorphsList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
                Children::spawn(SpawnIter(items.into_iter())),
            ),
            (
                Name::new("AddMorphButton"),
                MorphAddButton,
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new("+ Add Morph"), ThemedText)),
                ),
                observe(on_open_morph_menu(human_entity)),
            ),
        ],
        observe(on_morph_select(human_entity)),
        observe(on_morph_remove(human_entity)),
        observe(on_morph_value_change(human_entity)),
        observe(on_morph_close),
        observe(on_morph_filter),
    )
}

#[derive(Component)]
struct MorphsSection;

#[derive(Component)]
struct MorphsList;

#[derive(Component)]
struct MorphAddButton;

#[derive(EntityEvent)]
struct MorphSelect {
    entity: Entity,
    target: MorphTarget,
}

#[derive(EntityEvent)]
struct MorphRemove {
    entity: Entity,
    idx: usize,
}

#[derive(EntityEvent)]
struct MorphValueChange {
    entity: Entity,
    idx: usize,
    value: f32,
}

#[derive(EntityEvent)]
struct MorphClose {
    entity: Entity,
}


/// Single morph row with label, slider, and remove button
fn morph_item_row(_human_entity: Entity, idx: usize, morph: &Morph) -> impl Bundle {
    let (min, max) = morph.target.value_range();
    let label = format!("{:?}", morph.target);

    (
        Name::new(format!("MorphItem_{}", idx)),
        MorphItem(idx),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            width: Val::Percent(100.0),
            ..default()
        },
        children![
            (
                Text::new(label),
                TextFont { font_size: 10.0, ..default() },
                ThemedText,
                Node {
                    width: Val::Px(120.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
            ),
            (
                Node { flex_grow: 1.0, ..default() },
                children![(
                    slider(
                        SliderProps { value: morph.value, min, max },
                        (),
                    ),
                    observe(on_morph_slider_change),
                )],
            ),
            (
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    ..default()
                },
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new("×"), ThemedText)),
                    ),
                    observe(on_morph_item_remove_click),
                )],
            ),
        ],
    )
}

#[derive(Component)]
struct MorphItem(usize);

#[derive(Component)]
struct MorphMenu;

#[derive(Component)]
struct MorphOption(MorphTarget);

#[derive(Component)]
struct MorphFilterInput;

#[derive(Component)]
struct MorphOptionsContainer;

/// Slider change handler for morph items
fn on_morph_slider_change(
    trigger: On<ValueChange<f32>>,
    mut commands: Commands,
    parent_query: Query<&ChildOf>,
    item_query: Query<&MorphItem>,
    section_query: Query<&MorphsSection>,
) {
    // Update slider UI
    commands.entity(trigger.source).insert(SliderValue(trigger.value));

    // Find MorphItem parent to get index
    for ancestor in parent_query.iter_ancestors(trigger.source) {
        if let Ok(item) = item_query.get(ancestor) {
            // Find MorphsSection ancestor to trigger event
            for section_ancestor in parent_query.iter_ancestors(ancestor) {
                if section_query.get(section_ancestor).is_ok() {
                    commands.trigger(MorphValueChange {
                        entity: section_ancestor,
                        idx: item.0,
                        value: trigger.value,
                    });
                    return;
                }
            }
        }
    }
}

/// Handler for morph value change
fn on_morph_value_change(
    human_entity: Entity,
) -> impl FnMut(On<MorphValueChange>, Commands, Query<&mut Morphs>) {
    move |trigger: On<MorphValueChange>,
          mut commands: Commands,
          mut morphs_query: Query<&mut Morphs>| {
        if let Ok(mut morphs) = morphs_query.get_mut(human_entity) {
            if trigger.idx < morphs.len() {
                morphs[trigger.idx].value = trigger.value;
                commands.entity(human_entity).insert(HumanDirty);
            }
        }
    }
}

/// Click handler for remove button
fn on_morph_item_remove_click(
    trigger: On<Pointer<Click>>,
    mut commands: Commands,
    parent_query: Query<&ChildOf>,
    item_query: Query<&MorphItem>,
    section_query: Query<&MorphsSection>,
) {
    for ancestor in parent_query.iter_ancestors(trigger.entity) {
        if let Ok(item) = item_query.get(ancestor) {
            for section_ancestor in parent_query.iter_ancestors(ancestor) {
                if section_query.get(section_ancestor).is_ok() {
                    commands.trigger(MorphRemove {
                        entity: section_ancestor,
                        idx: item.0,
                    });
                    return;
                }
            }
        }
    }
}

/// Handler for MorphRemove event
fn on_morph_remove(
    human_entity: Entity,
) -> impl FnMut(On<MorphRemove>, Commands, Query<&mut Morphs>, Query<&Children>, Query<Entity, With<MorphsList>>) {
    move |trigger: On<MorphRemove>,
          mut commands: Commands,
          mut morphs_query: Query<&mut Morphs>,
          children_query: Query<&Children>,
          list_query: Query<Entity, With<MorphsList>>| {
        if let Ok(mut morphs) = morphs_query.get_mut(human_entity) {
            if trigger.idx < morphs.len() {
                morphs.remove(trigger.idx);
                commands.entity(human_entity).insert(HumanDirty);

                // Rebuild UI
                for child in children_query.iter_descendants(trigger.entity) {
                    if let Ok(list_entity) = list_query.get(child) {
                        commands.entity(list_entity).despawn_children();
                        for (idx, morph) in morphs.iter().enumerate() {
                            commands.entity(list_entity).with_child(
                                morph_item_row(human_entity, idx, morph)
                            );
                        }
                        break;
                    }
                }
            }
        }
    }
}

/// Handler for MorphSelect event
fn on_morph_select(
    human_entity: Entity,
) -> impl FnMut(On<MorphSelect>, Commands, Query<&mut Morphs>, Query<&Children>, Query<Entity, With<MorphsList>>) {
    move |trigger: On<MorphSelect>,
          mut commands: Commands,
          mut morphs_query: Query<&mut Morphs>,
          children_query: Query<&Children>,
          list_query: Query<Entity, With<MorphsList>>| {
        // Close menu
        commands.trigger(MorphClose {
            entity: trigger.entity,
        });

        if let Ok(mut morphs) = morphs_query.get_mut(human_entity) {
            // Check if already has this morph
            if !morphs.iter().any(|m| m.target == trigger.target) {
                let new_morph = Morph::new(trigger.target, 0.0);
                morphs.push(new_morph.clone());
                commands.entity(human_entity).insert(HumanDirty);

                // Add to UI
                for child in children_query.iter_descendants(trigger.entity) {
                    if let Ok(list_entity) = list_query.get(child) {
                        let idx = morphs.len() - 1;
                        commands.entity(list_entity).with_child(
                            morph_item_row(human_entity, idx, &new_morph)
                        );
                        break;
                    }
                }
            }
        }
    }
}

/// Handler for MorphClose event
fn on_morph_close(
    trigger: On<MorphClose>,
    mut commands: Commands,
    children_query: Query<&Children>,
    menu_query: Query<&MorphMenu>,
) {
    for child in children_query.iter_descendants(trigger.entity) {
        if menu_query.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

/// Handler for morph filter event
fn on_morph_filter(
    trigger: On<FilterOptions>,
    children_query: Query<&Children>,
    options_container: Query<Entity, With<MorphOptionsContainer>>,
    mut query: Query<(&MorphOption, &mut Node)>,
) {
    for child in children_query.iter_descendants(trigger.entity) {
        if let Ok(container) = options_container.get(child) {
            for c in children_query.iter_descendants(container) {
                if let Ok((option, mut node)) = query.get_mut(c) {
                    let label = format!("{:?}", option.0).to_lowercase();
                    let filter = trigger.filter.to_lowercase();
                    let show = filter.is_empty() || label.contains(&filter);
                    node.display = if show { Display::Flex } else { Display::None };
                }
            }
            break;
        }
    }
}

/// Open morph menu with all categories
fn on_open_morph_menu(
    human_entity: Entity,
) -> impl FnMut(On<Pointer<Click>>, Commands, Query<&Children>, Query<&MorphMenu>, Query<&ChildOf>, Query<&MorphsSection>) {
    move |trigger: On<Pointer<Click>>,
          mut commands: Commands,
          children_query: Query<&Children>,
          menu_query: Query<&MorphMenu>,
          parent_query: Query<&ChildOf>,
          section_query: Query<&MorphsSection>| {
        // Find MorphsSection ancestor
        let section_entity = parent_query
            .iter_ancestors(trigger.entity)
            .find(|e| section_query.get(*e).is_ok())
            .unwrap_or(trigger.entity);

        // Check if menu already open
        for child in children_query.iter_descendants(section_entity) {
            if menu_query.get(child).is_ok() {
                return;
            }
        }

        // Build morph options from all categories
        let mut options: Vec<_> = Vec::new();

        // Add all morph targets from each category
        for arms in ArmsMorph::iter() {
            options.push(MorphTarget::Arms(arms));
        }
        for breast in BreastMorph::iter() {
            options.push(MorphTarget::Breast(breast));
        }
        for buttocks in ButtocksMorph::iter() {
            options.push(MorphTarget::Buttocks(buttocks));
        }
        for cheek in CheekMorph::iter() {
            options.push(MorphTarget::Cheek(cheek));
        }
        for chin in ChinMorph::iter() {
            options.push(MorphTarget::Chin(chin));
        }
        for ears in EarsMorph::iter() {
            options.push(MorphTarget::Ears(ears));
        }
        for eyebrows in EyebrowsMorph::iter() {
            options.push(MorphTarget::Eyebrows(eyebrows));
        }
        for eyes in EyesMorph::iter() {
            options.push(MorphTarget::Eyes(eyes));
        }
        for feet in FeetMorph::iter() {
            options.push(MorphTarget::Feet(feet));
        }
        for forehead in ForeheadMorph::iter() {
            options.push(MorphTarget::Forehead(forehead));
        }
        for genitals in GenitalsMorph::iter() {
            options.push(MorphTarget::Genitals(genitals));
        }
        for hands in HandsMorph::iter() {
            options.push(MorphTarget::Hands(hands));
        }
        for head in HeadMorph::iter() {
            options.push(MorphTarget::Head(head));
        }
        for hip in HipMorph::iter() {
            options.push(MorphTarget::Hip(hip));
        }
        for legs in LegsMorph::iter() {
            options.push(MorphTarget::Legs(legs));
        }
        for mouth in MouthMorph::iter() {
            options.push(MorphTarget::Mouth(mouth));
        }
        for neck in NeckMorph::iter() {
            options.push(MorphTarget::Neck(neck));
        }
        for nose in NoseMorph::iter() {
            options.push(MorphTarget::Nose(nose));
        }
        for pelvis in PelvisMorph::iter() {
            options.push(MorphTarget::Pelvis(pelvis));
        }
        for stomach in StomachMorph::iter() {
            options.push(MorphTarget::Stomach(stomach));
        }
        for torso in TorsoMorph::iter() {
            options.push(MorphTarget::Torso(torso));
        }

        let option_bundles: Vec<_> = options
            .into_iter()
            .map(|target| {
                let label = format!("{:?}", target);
                (
                    Name::new(label.clone()),
                    MorphOption(target),
                    Node {
                        width: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        min_height: Val::Px(24.0),
                        ..default()
                    },
                    children![(
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((
                                Text::new(label),
                                ThemedText,
                                TextFont { font_size: 10.0, ..default() },
                            )),
                        ),
                        observe(on_morph_option_click(human_entity, target)),
                    )],
                )
            })
            .collect();

        commands.entity(section_entity).with_child((
            MorphMenu,
            Name::new("MorphMenu"),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            },
            ThemeBackgroundColor(tokens::WINDOW_BG),
            ZIndex(10),
            children![
                (
                    MorphFilterInput,
                    text_input(
                        TextInputProps {
                            width: Val::Percent(100.0),
                            height: Val::Px(24.0),
                            placeholder: "Filter...".to_string(),
                            corners: RoundedCorners::Top,
                            ..default()
                        },
                        TextInputContents::default(),
                    ),
                ),
                scroll(
                    ScrollProps::vertical(px(300.0)),
                    (MorphOptionsContainer, Name::new("MorphOptions")),
                    Children::spawn(SpawnIter(option_bundles.into_iter())),
                ),
            ],
            Hovered(false),
            observe(on_hover_exit),
        ));
    }
}

/// Click handler for morph option
fn on_morph_option_click(
    _human_entity: Entity,
    target: MorphTarget,
) -> impl FnMut(On<Pointer<Click>>, Commands, Query<&ChildOf>, Query<&MorphsSection>) {
    move |trigger: On<Pointer<Click>>,
          mut commands: Commands,
          parent_query: Query<&ChildOf>,
          section_query: Query<&MorphsSection>| {
        if let Some(section) = parent_query
            .iter_ancestors(trigger.entity)
            .find(|e| section_query.get(*e).is_ok())
        {
            commands.trigger(MorphSelect {
                entity: section,
                target,
            });
        }
    }
}

// ==================== END MORPHS SECTION ====================

/// Event for selecting an option from the dropdown
#[derive(EntityEvent)]
pub struct DropdownSelect<T: Component + Copy + Send + Sync + 'static> {
    entity: Entity,
    value: T,
}

/// Event for closing the dropdown
#[derive(EntityEvent)]
pub struct DropdownClose {
    entity: Entity,
}

/// Event for filtering dropdown options
#[derive(EntityEvent)]
pub struct FilterOptions {
    entity: Entity,
    filter: String,
}

/// Marker for dropdowns    
#[derive(Component)]
pub struct Dropdown;

/// Marker for dropdown button
#[derive(Component)]
pub struct DropdownButton;

/// Marker for open dropdown menu
#[derive(Component)]
pub struct DropdownMenu;

/// Marker for dropdown filter text input
#[derive(Component)]
pub struct DropdownFilterInput;

/// Marker for dropdown options container
#[derive(Component)]
pub struct DropdownOptionsContainer;

/// Dropdown without thumbnails (for types without MHThumb like Rig)
fn dropdown_mh<T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static>(
    human_entity: Entity,
    value: T,
) -> impl Bundle {
    let type_name = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap_or_default();

    (
        Name::new(format!("Dropdown{}", type_name)),
        Dropdown,
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Text::new(type_name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                ThemedText
            ),
            (
                Name::new("DropdownButton"),
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new(value.to_string()), ThemedText,)),
                ),
                observe(on_open_dropdown::<T>), // only line different from thumbnail version
            ),
        ],
        observe(on_dropdown_select::<T>(human_entity)),
        observe(on_dropdown_close),
        observe(on_dropdown_filter::<T>),
    )
}


fn dropdown_mh_thumb<
    T: Component + Copy + IntoEnumIterator + ToString + MHThumb + Send + Sync + 'static,
>(
    human_entity: Entity,
    value: T,
) -> impl Bundle {
    let type_name = std::any::type_name::<T>()
        .split("::")
        .last()
        .unwrap_or_default();

    (
        Dropdown,
        Name::new(format!("Dropdown {}", type_name)),
        Node {
            padding: UiRect::top(Val::Px(8.0)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![
            (
                Text::new(type_name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                ThemedText
            ),
            (
                Name::new("DropdownButton"),
                DropdownButton,
                button(
                    ButtonProps::default(),
                    (),
                    Spawn((Text::new(value.to_string()), ThemedText,)),
                ),
                observe(on_open_dropdown_thumb::<T>), // only line diff
            ),
        ],
        observe(on_dropdown_select::<T>(human_entity)),
        observe(on_dropdown_close),
        observe(on_dropdown_filter::<T>),
    )
}

fn on_dropdown_select<T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static>(
    human_entity: Entity,
) -> impl FnMut(On<DropdownSelect<T>>, Commands, Query<&DropdownButton>, Query<&Children>) {
    move |trigger: On<DropdownSelect<T>>, mut commands: Commands, dropdown_button_query: Query<&DropdownButton>, children_query: Query<&Children>| {
        info!("select");
        // close dropdown
        commands.trigger(DropdownClose {
            entity: trigger.entity,
        });

        // set component on human entity
        commands.entity(human_entity).insert(trigger.value);
        
        // update button label
        for e in children_query.iter_descendants(trigger.entity) {
            if dropdown_button_query.get(e).is_ok() {
                commands.entity(e).despawn_children().with_child((
                    Text::new(trigger.value.to_string()),
                    ThemedText,
                ));
                break;
            }
        }
    }
}

fn on_option_click<T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static>(
    value: T,
) -> impl FnMut(On<Pointer<Click>>, Commands, Query<&ChildOf>, Query<&Dropdown>) {
    move |trigger: On<Pointer<Click>>,
          mut commands: Commands,
          parent_query: Query<&ChildOf>,
          dropdown_query: Query<&Dropdown>| {
        let parent = parent_query
            .iter_ancestors(trigger.entity)
            .find(|c| dropdown_query.get(*c).is_ok())
            .unwrap();
        commands.trigger(DropdownSelect {
            entity: parent,
            value: value,
        });
    }
}

fn on_dropdown_close(
    trigger: On<DropdownClose>,
    mut commands: Commands,
    children_query: Query<&Children>,
    dropdown_open: Query<&DropdownMenu>,
) {
    for child in children_query.get(trigger.entity).unwrap().iter() {
        if dropdown_open.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }
}

fn on_dropdown_filter<T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static>(
    trigger: On<FilterOptions>,
    children_query: Query<&Children>,
    dropdown_options_container: Query<Entity, With<DropdownOptionsContainer>>,
    mut query: Query<(&T, &mut Node)>,
) {
    for child in children_query.iter_descendants(trigger.entity) {
        if let Ok(container) = dropdown_options_container.get(child) {
            for c in children_query.iter_descendants(container) {
                if let Ok((value, mut node)) = query.get_mut(c) {
                    let label = value.to_string().to_lowercase();
                    let filter = trigger.filter.to_lowercase();
                    let show = filter.is_empty() || label.contains(&filter);
                    node.display = if show { Display::Flex } else { Display::None };
                }
            }
            break;
        }
    }
}

fn on_open_dropdown<T: Component + Copy + IntoEnumIterator + ToString + Send + Sync + 'static>(
    trigger: On<Pointer<Click>>,
    children_query: Query<&Children>,
    dropdown_open: Query<&DropdownMenu>,
    parent_query: Query<&ChildOf>,
    mut commands: Commands,
) {
    let child_of = parent_query.get(trigger.entity).unwrap();

    // check if already open
    for child in children_query.iter_descendants(child_of.0) {
        if dropdown_open.get(child).is_ok() {
            return;
        }
    }

    let options: Vec<_> = T::iter()
        .map(|value| {
            let label = value.to_string();
            (
                Name::new(label.clone()),
                Node {
                    width: Val::Percent(100.0),

                    min_height: Val::Px(28.0),
                    ..default()
                },
                value.clone(),
                children![(
                    button(
                        ButtonProps::default(),
                        (),
                        Spawn((Text::new(label), ThemedText))
                    ),
                    observe(on_option_click(value)),
                )],
            )
        })
        .collect();

    commands.entity(child_of.0).with_child((
        DropdownMenu,
        Name::new("DropdownOpen"),
        Node {
            padding: UiRect::top(Val::Px(4.0)),
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            ..default()
        },
        ThemeBackgroundColor(tokens::BUTTON_BG),
        children![
            (
                DropdownFilterInput,
                text_input(
                    TextInputProps {
                        width: Val::Percent(100.0),                        
                        height: Val::Px(24.0),
                        placeholder: "Filter...".to_string(),
                        corners: RoundedCorners::Top,
                        ..default()
                    },
                    TextInputContents::default()
                ),
            ),
            scroll(
                ScrollProps::vertical(px(400.0)),
                (DropdownOptionsContainer, Name::new("OptionsContainer")),
                Children::spawn(SpawnIter(options.into_iter())),
            ),
        ],
        Hovered(false),
        observe(on_hover_exit),
    ));
}


/// Dropdown with thumbnails (for types with MHThumb)
fn on_open_dropdown_thumb<T: Component + Copy + IntoEnumIterator + ToString + MHThumb + Send + Sync + 'static>(
    trigger: On<Pointer<Click>>,
    children_query: Query<&Children>,
    dropdown_open: Query<&DropdownMenu>,
    parent_query: Query<&ChildOf>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let child_of = parent_query.get(trigger.entity).unwrap();

    // check if already open
    for child in children_query.iter_descendants(child_of.0) {
        if dropdown_open.get(child).is_ok() {
            return;
        }
    }

    let options: Vec<_> = T::iter()
        .map(|value| {
            let label = value.to_string();
            let image = asset_server.load::<Image>(value.thumb());
            (
                Name::new(label.clone()),
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    min_height: Val::Px(28.0),
                    ..default()
                },
                value.clone(),
                children![
                    (
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            ..default()
                        },
                        ImageNode{
                            image: image,   
                            ..default()
                        }
                    ),
                    (
                        button(
                            ButtonProps::default(),
                            (),
                            Spawn((Text::new(label), ThemedText))
                        ),
                        observe(on_option_click(value)),
                    )
                ],
            )
        })
        .collect();

    commands.entity(child_of.0).with_child((
        DropdownMenu,
        Name::new("DropdownOpen"),
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            ..default()
        },
        ThemeBackgroundColor(tokens::WINDOW_BG),
        children![
            (
                DropdownFilterInput,
                text_input(
                    TextInputProps {
                        width: Val::Percent(100.0),
                        height: Val::Px(24.0),
                        placeholder: "Filter...".to_string(),
                        corners: RoundedCorners::Top,
                        ..default()
                    },
                    TextInputContents::default()
                ),
            ),
            scroll(
                ScrollProps::vertical(px(400.0)),
                (DropdownOptionsContainer, Name::new("OptionsContainer")),
                Children::spawn(SpawnIter(options.into_iter())),
            ),
        ],
        Hovered(false),
        observe(on_hover_exit),
    ));
}


/// Close dropdown when hover exits
fn on_hover_exit(trigger: On<Pointer<Out>>, mut commands: Commands, hover_query: Query<&Hovered>) {    
    if let Ok(hovered) = hover_query.get(trigger.entity) {
        if !hovered.0 {
            commands.entity(trigger.entity).despawn();
        }
    }
}

/// Filter options based on text input (for dropdowns)
fn filter_text_changed<T: Component, K: Component>(
    filter_query: Query<
        (Entity, &TextInputContents),
        (With<T>, Changed<TextInputContents>),
    >,
    
    parent_query: Query<&ChildOf>,
    dropdown_query: Query<&K>,
    mut commands: Commands,
) {
    // Handle dropdown filters
    for (e, text) in filter_query.iter() {
        for c in parent_query.iter_ancestors(e) {
            if dropdown_query.contains(c) {
                commands.trigger(FilterOptions {
                    entity: c,
                    filter: text.get().to_string(),
                });
            }
        }
    }
}

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
