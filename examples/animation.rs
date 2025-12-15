#![allow(warnings)]
#[path = "common/mod.rs"]
mod common;
use common::*;

use avian3d::prelude::*;
use bevy::{
    animation::{AnimationTargetId, animated_field},
    platform::collections::HashMap,
    prelude::*,
};
use bevy_asset_loader::prelude::*;
use bevy_make_human::prelude::*;
use std::{any::TypeId, f32::consts::PI};

fn main() -> AppExit {
    todo!("wip");
    // App::ne1w()
    //     .add_plugins((
    //         DefaultPlugins,
    //         PhysicsPlugins::default(),
    //         MakeHumanPlugin::default(),
    //         CommonPlugin, // camera controls, egui, mipmaps, skinned AABB
    //     ))
    //     .init_collection::<MixamoAnimations>()
    //     .add_systems(Startup, setup)
    //     .run()
}

#[derive(AssetCollection, Resource)]
struct MixamoAnimations {
    #[asset(path = "animations/mixamo/Warrior Idle.glb")]
    pub warrior_idle: Handle<Gltf>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CameraFree::default(), // camera controller
        Camera3d::default(),
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
            Outfit(vec![
                Clothing::ToigoMaleSuit3,
                Clothing::ToigoAnkleBootsMale,
            ]),
            Morphs(vec![Morph::new(
                MorphTarget::Macro(MacroMorph::CaucasianMaleYoung),
                1.0,
            )]),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .observe(apply_gltf_animation);
}

#[allow(dead_code)]
fn apply_gltf_animation(
    trigger: On<HumanComplete>,
    mut human_query: Query<(&Rig, &mut Skeleton), With<Human>>,
    mut commands: Commands,
    assets: Res<MixamoAnimations>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<bevy::gltf::GltfNode>>,
    children_query: Query<&Children>,
    mut animation_player_query: Query<&mut AnimationPlayer, With<Armature>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Get config and skeleton
    let (rig, skeleton) = human_query.get_mut(trigger.entity).unwrap();

    // For Mixamo-compatible rigs, build direct mapping
    if rig != &Rig::Mixamo {
        error!("GLTF animation retargeting only implemented for Mixamo rigs right now");
        return;
    }

    // Get the GLTF asset
    let gltf = gltfs.get(&assets.warrior_idle).unwrap();
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

        info!(
            "Retargeted {} targets, {} unmapped, {} scale curves skipped",
            curves_copied, unmapped, scale_skipped
        );

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

//
// Animations Tests
//

// Apply a custom animation that moves multiple bones
#[allow(dead_code)]
fn apply_custom_animation(
    trigger: On<HumanComplete>,
    mut commands: Commands,
    children_query: Query<&Children>,
    mut animation_player_query: Query<&mut AnimationPlayer>,
    skeleton_query: Query<&Skeleton>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    error!("Custom animation applied");

    // Get skeleton to find bone names
    let Ok(skeleton) = skeleton_query.get(trigger.entity) else {
        warn!("No skeleton for animation setup");
        return;
    };

    // Find rig entity (has AnimationPlayer)
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

    // Create animation clip with rotation keyframes for multiple bones
    let mut clip = AnimationClip::default();

    // Animate several bones to test full body skinning
    let bones_to_animate = [
        ("upperarm_l", Quat::from_rotation_x(PI / 4.0)),
        ("upperarm_r", Quat::from_rotation_x(-PI / 4.0)),
        ("thigh_l", Quat::from_rotation_x(PI / 6.0)),
        ("thigh_r", Quat::from_rotation_x(-PI / 6.0)),
        ("spine_02", Quat::from_rotation_y(PI / 8.0)),
        ("head", Quat::from_rotation_y(PI / 6.0)),
    ];

    for (bone_name, rotation) in bones_to_animate {
        let Some(bone_idx) = skeleton.bone_index(bone_name) else {
            continue;
        };

        // Build animation target path: bone -> ... -> root -> Rig
        let mut path = vec![Name::new(bone_name.to_string())];
        let mut current_idx = bone_idx;
        while let Some(parent_idx) = skeleton.hierarchy[current_idx] {
            path.push(Name::new(skeleton.bones[parent_idx].name.clone()));
            current_idx = parent_idx;
        }
        path.push(Name::new("Rig"));

        let target_id = AnimationTargetId::from_names(path.iter().rev());

        clip.add_curve_to_target(
            target_id,
            AnimatableCurve::new(
                animated_field!(Transform::rotation),
                UnevenSampleAutoCurve::new([0.0, 1.0, 2.0].into_iter().zip([
                    Quat::IDENTITY,
                    rotation,
                    Quat::IDENTITY,
                ]))
                .expect("valid curve"),
            ),
        );
    }

    let clip_handle = animation_clips.add(clip);

    // Create animation graph
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);

    // Attach graph to rig and play
    commands
        .entity(rig)
        .insert(AnimationGraphHandle(graph_handle));

    if let Ok(mut player) = animation_player_query.get_mut(rig) {
        player.play(node_index).repeat();
        info!("Started multi-bone test animation on rig");
    }
}
