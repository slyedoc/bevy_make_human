pub mod assets;
pub mod components;
#[cfg(feature = "debug_draw")]
pub mod debug_draw;
pub mod events;
pub mod loaders;
pub mod skeleton;
pub mod util;

use crate::{assets::*, components::*, events::*, loaders::*, skeleton::*, util::*};

pub mod prelude {
    #[cfg(feature = "debug_draw")]
    pub use crate::debug_draw::*;
    #[allow(unused_imports)]
    pub use crate::{
        MHState, MakeHumanPlugin, assets::*, components::*, events::*, loaders::*, skeleton::*,
        util::*,
    };
}

use bevy::{
    mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
use bevy_asset_loader::prelude::*;
use avian3d::prelude::*;

#[derive(Default, States, Debug, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum MHState {
    #[default]
    LoadingAssets,
    LoadingBasemesh,
    Ready,
}

#[derive(Default)]
pub struct MakeHumanPlugin;

impl Plugin for MakeHumanPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_obj::ObjPlugin,
            #[cfg(feature = "debug_draw")]
            debug_draw::MakeHumanDebugPlugin,
        ))
        .init_state::<MHState>()
        .add_loading_state(
            LoadingState::new(MHState::LoadingAssets)
                .load_collection::<BaseMeshAssets>()
                .continue_to_state(MHState::LoadingBasemesh),
        )
        // TODO: save this work out instead of rebuilding every time
        .add_systems(OnEnter(MHState::LoadingBasemesh), build_basemesh)
        .add_systems(
            Update,
            poll_basemesh_task.run_if(in_state(MHState::LoadingBasemesh)),
        )
        // Steps:
        // 1. On load or change, load needed assets,
        // 2. Once loaded, generate new assets(mesh) in async task        
        .add_systems(OnEnter(MHState::Ready), init_existing_character)
        .add_systems(
            Update,
            (character_changed, character_loading, update_character)
                .run_if(in_state(MHState::Ready)),
        );

        // asset loaders
        app
            // base mesh .obj loader with original verts
            .init_asset::<ObjBaseMesh>()
            .init_asset_loader::<ObjBaseMeshLoader>()
            // vertex groups loader
            .init_asset::<VertexGroups>()
            .init_asset_loader::<VertexGroupsLoader>()
            // morph target loader
            .init_asset::<MorphTargetData>()
            .init_asset_loader::<MorphTargetLoader>()
            // // faceshapes (.mxa) loader for FACS expressions
            // .init_asset::<FaceshapesData>()
            // .init_asset_loader::<FaceshapesLoader>()
            // mhclo loader
            .init_asset::<MhcloAsset>()
            .init_asset_loader::<MhcloLoader>()
            // proxy mesh loader
            .init_asset::<ProxyAsset>()
            .init_asset_loader::<ProxyLoader>()
            // bones
            .init_asset::<RigBones>()
            .init_asset_loader::<RigLoader>()
            // skinning weights
            .init_asset::<SkinningWeights>()
            .init_asset_loader::<SkinningWeightsLoader>()
            // mhmat to material loader
            .init_asset_loader::<MhmatLoader>() // -> StandardMaterial
            // thumb image loader (PNG thumbnails)
            .init_asset_loader::<ThumbLoader>() // -> Image
            // bvh pose loader
            .init_asset::<Pose>()
            .init_asset_loader::<BvhPoseLoader>()
            // egui registration
            .register_type::<HumanConfig>()
            .register_type::<Morph>();
    }
}

// TODO: from Humentity, leaving here as reminder to checkout later, tried scaling tracks myself
/// Use animation postprocessing to rescale position tracks to mesh size
/// This has some performance overhead. If disabled then translation tracks will
/// be removed from all retargeted animations.
// #[derive(Copy, Clone, Default, Resource, Debug)]
// pub enum TranslationTracks {
//     #[default]
//     Root,
//     Full,
//     None,
// }

#[derive(AssetCollection, Resource)]
pub struct BaseMeshAssets {
    #[asset(path = "make_human/3dobjs/base.obj")]
    pub obj: Handle<ObjBaseMesh>,
    #[asset(path = "make_human/mesh_metadata/basemesh.vertex_groups.json")]
    pub vertex_groups: Handle<VertexGroups>,
    // #[asset(path = "make_human/mesh_metadata/hm08_config.json")]
    // pub config: Handle<BasemeshConfig>,
}

#[derive(Resource, Default)]
pub struct BaseMesh {
    pub mesh: Handle<Mesh>,
    /// The vertices in the base mesh
    pub vertices: Vec<Vec3>,
    /// Maps Bevy mesh vertex idx -> MH obj vertex idx (handles UV seam duplicates)
    pub mhid_lookup: Vec<u16>,
    /// Vertex groups for bone CUBE/MEAN strategies
    pub vertex_groups: VertexGroups,
}

#[derive(Resource)]
pub struct PrepareBasemeshTask(Task<PrepareBasemeshOutput>);

pub struct PrepareBasemeshOutput {
    pub mhid_lookup: Vec<u16>,
}

fn build_basemesh(
    mut commands: Commands,
    base_mesh_assets: Res<BaseMeshAssets>,
    obj_assets: ResMut<Assets<ObjBaseMesh>>,
) {
    // grab copy for async task
    let obj_base_mesh = obj_assets.get(&base_mesh_assets.obj).unwrap().clone();

    let task = AsyncComputeTaskPool::get().spawn(async move {
        
        // Get mesh arrays and build mhid_lookup, takes 220ms
        let vtx_data = get_vertex_positions(&obj_base_mesh.mesh);
        let vertex_map = generate_vertex_map(&obj_base_mesh.vertices, &vtx_data);
        let mhid_lookup = generate_mhid_lookup(&vertex_map);

        PrepareBasemeshOutput {
            mhid_lookup,
        }
    });
    commands.insert_resource(PrepareBasemeshTask(task));
}

fn poll_basemesh_task(
    mut commands: Commands,
    base_mesh_assets: Res<BaseMeshAssets>,
    obj_assets: ResMut<Assets<ObjBaseMesh>>,
    vg_assets: Res<Assets<VertexGroups>>,
    mut prepare_task: ResMut<PrepareBasemeshTask>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(PrepareBasemeshOutput {
        mhid_lookup,
    }) = future::block_on(future::poll_once(&mut prepare_task.0))
    {
        let obj_base_mesh = obj_assets
            .get(&base_mesh_assets.obj)
            .expect("Basemesh ojb loaded")
            .clone();

        let vg = vg_assets
            .get(&base_mesh_assets.vertex_groups)
            .expect("vg loaded")
            .clone();

        commands.insert_resource(BaseMesh {
            mesh: meshes.add(obj_base_mesh.mesh.clone()),
            vertices: obj_base_mesh.vertices.clone(),
            mhid_lookup,
            vertex_groups: vg.clone(),
            ..default()
        });
        commands.remove_resource::<BaseMeshAssets>();
        commands.set_state(MHState::Ready);
    }
}

/// Task component for async character processing
#[derive(Component)]
pub struct CharacterProcessingTask(Task<CharacterProcessingOutput>);


/// All data needed for character processing (extracted from assets)
struct CharacterProcessingInput {
    base_vertices: Vec<Vec3>,
    vertex_groups: VertexGroups,
    // Morphs
    phenotype_morphs: Vec<(MorphTargetData, f32)>,
    regular_morphs: Vec<(MorphTargetData, f32)>,
    // Rig
    rig_bones: RigBones,
    skinning_weights: SkinningWeights,

    // Skin Proxy
    proxy_asset: ProxyAsset,
    proxy_obj_base: ObjBaseMesh,
    skin: Handle<StandardMaterial>,
    
    // Parts
    parts: Vec<MHItemLoaded>,
    
    clothing_offset: f32,
}

/// Result of character processing
struct CharacterProcessingOutput {
    skeleton: Skeleton,    
    parts: Vec<MHItemResult>,
    /// Character height (max_y - min_y of morphed vertices)
    height: f32,
    /// Min Y of morphed vertices (for ground offset)
    min_y: f32,
}

fn character_changed(
    mut commands: Commands,
    query: Query<
        (Entity, &HumanConfig, &Phenotype),
        Or<(Changed<HumanConfig>, Changed<Phenotype>)>,
    >,
    asset_server: Res<AssetServer>,
) {
    for (e, config, phenotype) in query.iter() {
        commands
            .entity(e)
            // Cancel any existing processing task
            .remove::<CharacterProcessingTask>()
            // create a entity level asset collection
            .insert(CharacterAssets::from_config(
                config,
                phenotype,
                &asset_server,
            ));
    }
}

fn init_existing_character(
    mut commands: Commands,
    query: Query<(Entity, &HumanConfig, &Phenotype), Without<CharacterAssets>>,
    asset_server: Res<AssetServer>,
) {
    for (e, config, phenotype) in query.iter() {
        info!("Initializing existing character {:?}", e);
        commands
            .entity(e) // create a entity level asset collection
            .insert(CharacterAssets::from_config(
                config,
                phenotype,
                &asset_server,
            ));
    }
}

fn character_loading(
    mut commands: Commands,
    mut query: Query<(Entity, &CharacterAssets, Option<&Name>)>,
    asset_server: Res<AssetServer>,
    base_mesh: Res<BaseMesh>,
    mhclo_assets: Res<Assets<MhcloAsset>>,
    proxy_assets: Res<Assets<ProxyAsset>>,
    obj_base_assets: Res<Assets<ObjBaseMesh>>,
    rig_bones_assets: Res<Assets<RigBones>>,
    skinning_weights_assets: Res<Assets<SkinningWeights>>,
    morph_target_assets: Res<Assets<MorphTargetData>>,
) {
    for (e, assets, name) in query.iter_mut() {
        let handles = assets.all_handles();
        let total = handles.len();
        let loaded = handles
            .iter()
            .filter(|h| asset_server.is_loaded_with_dependencies(h.id()))
            .count();

        info!(
            "[{:?}] Loaded {}/{} assets",
            name.map(|n| n.as_str()).unwrap_or("Character"),
            loaded,
            total
        );

        if loaded >= total {
            let char_name = name.map(|n| n.as_str()).unwrap_or("Character");
            info!(
                "[{}] All assets loaded, spawning processing task",
                char_name
            );

            let parts = assets
                .parts
                .iter()
                .map(|a| MHItemLoaded {
                        tag: a.tag,
                        base:  obj_base_assets.get(&a.obj_base).unwrap().clone(),
                        mat: a.mat.clone(),
                        clo:  mhclo_assets.get(&a.clo).unwrap().clone(),
                    })
                .collect::<Vec<_>>();
            
            // Extract for task
            let input = CharacterProcessingInput {
                base_vertices: base_mesh.vertices.clone(),
                vertex_groups: base_mesh.vertex_groups.clone(),
                phenotype_morphs: assets
                    .phenotype_morphs
                    .iter()
                    .filter_map(|(h, w)| morph_target_assets.get(h).map(|m| (m.clone(), *w)))
                    .collect(),
                regular_morphs: assets
                    .morphs
                    .iter()
                    .filter_map(|(h, w)| morph_target_assets.get(h).map(|m| (m.clone(), *w)))
                    .collect(),
                rig_bones: rig_bones_assets.get(&assets.rig_bones).unwrap().clone(),
                skinning_weights: skinning_weights_assets
                    .get(&assets.rig_weights)
                    .unwrap()
                    .clone(),
                skin: assets.skin_material.clone(),
                proxy_asset: proxy_assets.get(&assets.proxy_proxy).unwrap().clone(),
                proxy_obj_base: obj_base_assets.get(&assets.proxy_obj_base).unwrap().clone(),
                clothing_offset: assets.clothing_offset,
                parts,
            };

            // Spawn async task
            let task = AsyncComputeTaskPool::get().spawn(async move { process_character(input) });
            commands
                .entity(e)
                .remove::<CharacterAssets>()
                .insert(CharacterProcessingTask(task));
        }
    }
}


fn process_character(input: CharacterProcessingInput) -> CharacterProcessingOutput {
    let mut morphed_vertices = input.base_vertices.clone();

    // Apply phenotype morphs
    for (morph_data, weight) in &input.phenotype_morphs {
        if *weight < 0.001 {
            continue;
        }
        for (&mh_idx, &offset) in &morph_data.offsets {
            let idx = mh_idx as usize;
            if idx < morphed_vertices.len() {
                morphed_vertices[idx] += offset * *weight;
            }
        }
    }

    // Apply regular morphs
    for (morph_data, value) in &input.regular_morphs {
        let weight = value.abs();
        for (&mh_idx, &offset) in &morph_data.offsets {
            let idx = mh_idx as usize;
            if idx < morphed_vertices.len() {
                morphed_vertices[idx] += offset * weight;
            }
        }
    }

    // Build skeleton
    let skeleton = input.rig_bones.build_skeleton(&morphed_vertices, &input.vertex_groups);
    
    let mut parts = input.parts.into_iter().map(|s| MHItemResult {
        tag: s.tag,
        mesh: {
            let mesh = apply_mhclo_fitting(
                &s.base.mesh,
                &s.clo,
                &s.base.mhid_lookup,
                &morphed_vertices,
                match s.tag {
                    MHTag::Clothes => input.clothing_offset,
                    _ => 0.0
                }                
            );
            apply_skinning_weights_via_mhclo(
                mesh,
                &s.clo,
                &s.base.mhid_lookup,
                &skeleton,
                &input.skinning_weights,
            )
        },
        mat: s.mat,
    }).collect::<Vec<_>>();
    
    // Skin mesh from proxy
    let mut proxy_mesh = apply_proxy_fitting(
        &input.proxy_obj_base.mesh,
        &input.proxy_asset,
        &morphed_vertices,
        &input.proxy_obj_base.vertices,
    );
    proxy_mesh = apply_skinning_weights_to_proxy(
        proxy_mesh,
        &input.proxy_asset,
        &input.proxy_obj_base.mhid_lookup,
        &skeleton,
        &input.skinning_weights,
    );    
    parts.push(MHItemResult {
        tag: MHTag::Skin,
        mesh: proxy_mesh,
        mat: input.skin.clone(),
    });
    
    // Calculate character height from morphed vertices
    let (min_y, max_y) = morphed_vertices
        .iter()
        .fold((f32::MAX, f32::MIN), |(min, max), v| {
            (min.min(v.y), max.max(v.y))
        });
    let height = max_y - min_y;

    CharacterProcessingOutput {
        skeleton,
        parts,
        height,
        min_y,
    }
}

/// Poll processing tasks and trigger CharacterGenerate when done
fn update_character(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut CharacterProcessingTask,
        &HumanConfig,
    )>,
    children_query: Query<&Children>,
    character_part_query: Query<&MHTag>,
    mut inverse_bindpose_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task, config) in query.iter_mut() {
        if let Some( CharacterProcessingOutput { 
            skeleton,            
            parts,
            height,
            min_y,            
        }) = future::block_on(future::poll_once(&mut task.0)) {
            
            commands.entity(entity)
                .remove::<CharacterProcessingTask>();
                
            // Clean up previous character parts - collect first to avoid iterator invalidation
            let parts_to_despawn: Vec<Entity> = children_query
                .iter_descendants(entity)
                .filter(|&child| character_part_query.get(child).is_ok())
                .collect();
            for e in parts_to_despawn {
                commands.entity(e).try_despawn();
            }

            // Spawn skeleton bones and get joint entities
            let (_rig_entity, bone_entities) = spawn_skeleton_bones(&skeleton, entity, &mut commands);
        
            // Create SkinnedMesh component - shared by body and all parts
            let inverse_bindposes = inverse_bindpose_assets.add(skeleton.inverse_bind_matrices.clone());
            let skinned_mesh = SkinnedMesh {
                inverse_bindposes,
                joints: bone_entities.clone(),
            };
        
            // Capsule collider sized to character
            // Total capsule height = length + 2*radius (caps on each end)
            // Position so bottom of capsule aligns with min_y - floor_offset
            // (positive floor_offset = shoes, raises collider; negative = bare feet, lowers collider)
            let capsule_radius = 0.25;
            let capsule_length = (height - (capsule_radius * 2.0)).max(0.1);
            let collider = Collider::capsule(capsule_radius, capsule_length);
            // Center of capsule = min_y - floor_offset + radius + length/2
            let collider_center_y = min_y - config.floor_offset + capsule_radius + (capsule_length / 2.0);
            
            // Body mesh on main entity + faceshape deformation data
            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .insert(LockedAxes::ROTATION_LOCKED)
                .with_child((
                    Name::new("Collider"),
                    Transform::from_translation(Vec3::Y * collider_center_y),
                    collider,
                    MHTag::Collider,
                ));
            
            // parts
            for a in parts.into_iter() {
                commands.spawn((
                    ChildOf(entity),
                    Name::new(format!("{}", a.tag)),
                    Mesh3d(meshes.add(a.mesh)),
                    MeshMaterial3d(a.mat),
                    skinned_mesh.clone(),
                    a.tag,
                ));
            }
        
            // Notify character complete
            commands.trigger(CharacterComplete {
                entity
            });
            
           
        }
    }
}