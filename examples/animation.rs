//! Animation retargeting example for MakeHuman characters
//!
//! Demonstrates loading GLTF animations (e.g., Mixamo) and playing them
//! on procedurally generated MakeHuman skeletons.

#[path = "common/mod.rs"]
mod common;
use common::*;

use avian3d::prelude::*;
use bevy::{animation::AnimationTargetId, gltf::Gltf, platform::collections::HashMap, prelude::*};
use bevy_make_human::prelude::*;
use std::any::TypeId;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
            CommonPlugin, // camera controls, egui, mipmaps, skinned AABB
        ))
        .init_resource::<AnimationAssets>()
        .add_systems(Startup, (load_animations, setup))
        .add_systems(
            Update,
            setup_animation_graph.run_if(resource_exists::<AnimationAssets>),
        )
        .run()
}

#[derive(Resource, Default)]
struct AnimationAssets {
    gltf: Option<Handle<Gltf>>,
    graph: Option<Handle<AnimationGraph>>,
    node: Option<AnimationNodeIndex>,
}

fn load_animations(mut assets: ResMut<AnimationAssets>, asset_server: Res<AssetServer>) {
    // Load the Mixamo animation GLB
    assets.gltf = Some(asset_server.load("animations/mixamo/Breathing Idle.glb"));
}

fn setup_animation_graph(
    mut assets: ResMut<AnimationAssets>,
    gltfs: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<bevy::gltf::GltfNode>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Wait for GLTF to load
    let Some(gltf_handle) = &assets.gltf else {
        return;
    };
    let Some(gltf) = gltfs.get(gltf_handle) else {
        return;
    };

    // Only build graph once
    if assets.graph.is_some() {
        return;
    }

    if gltf.animations.is_empty() {
        warn!("No animations in GLTF");
        return;
    }

    // Build GLTF hierarchy paths from nodes
    let mut gltf_paths: HashMap<String, Vec<Name>> = HashMap::default();
    for node_handle in &gltf.nodes {
        if let Some(node) = gltf_nodes.get(node_handle) {
            if !gltf_paths.contains_key(&node.name) {
                build_gltf_paths(node_handle, &[], &gltf_nodes, &mut gltf_paths);
            }
        }
    }

    // Build source->target AnimationTargetId mapping
    // Mixamo GLTF uses "mixamorig:BoneName" which matches our Mixamo rig
    let mut id_map: HashMap<AnimationTargetId, AnimationTargetId> = HashMap::default();

    for (source_name, _) in &gltf.named_nodes {
        if let Some(source_path) = gltf_paths.get(source_name.as_ref()) {
            // Target path is the same as source for Mixamo->Mixamo
            let target_id = AnimationTargetId::from_names(source_path.iter());
            let source_id = AnimationTargetId::from_names(source_path.iter());
            id_map.insert(source_id, target_id);
        }
    }

    // Build animation graph with retargeted clips
    let mut graph = AnimationGraph::new();
    let mut first_node = None;

    // Transform::scale is field index 2 - skip scale curves from Mixamo (0.01 cm->m scaling)
    let transform_type_id = TypeId::of::<Transform>();
    const SCALE_FIELD_INDEX: usize = 2;

    for clip_handle in &gltf.animations {
        let Some(source_clip) = animation_clips.get(clip_handle) else {
            continue;
        };

        // Create retargeted clip - copy translation/rotation curves, skip scale
        let mut new_clip = AnimationClip::default();
        let mut curves_copied = 0;

        for (source_id, curves) in source_clip.curves().iter() {
            if let Some(&target_id) = id_map.get(source_id) {
                for curve in curves.iter() {
                    // Skip scale curves
                    let is_scale = match curve.0.evaluator_id() {
                        EvaluatorId::ComponentField(hashed) => {
                            let (type_id, field_idx) = **hashed;
                            type_id == transform_type_id && field_idx == SCALE_FIELD_INDEX
                        }
                        _ => false,
                    };

                    if !is_scale {
                        new_clip.add_variable_curve_to_target(target_id, curve.clone());
                        curves_copied += 1;
                    }
                }
            }
        }

        if curves_copied > 0 {
            let new_handle = animation_clips.add(new_clip);
            let node = graph.add_clip(new_handle, 1.0, graph.root);
            if first_node.is_none() {
                first_node = Some(node);
            }
            info!("Animation loaded with {} curves", curves_copied);
        }
    }

    assets.graph = Some(animation_graphs.add(graph));
    assets.node = first_node;
    info!("Animation graph ready");
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        CameraFree::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.5, -3.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

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

    // Spawn human with Mixamo rig (matches our animation skeleton)
    commands
        .spawn((
            Name::new("AnimatedHuman"),
            Human,
            Rig::Mixamo, // Must use Mixamo rig for Mixamo animations
            SkinMesh::MaleGeneric,
            SkinMaterial::YoungCaucasianMale,
            Eyes::LowPolyBluegreen,
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
        .observe(on_human_complete);
}

/// Called when the human is fully generated - attach animation graph and play
fn on_human_complete(
    trigger: On<HumanComplete>,
    assets: Res<AnimationAssets>,
    children: Query<&Children>,
    mut armature_query: Query<&mut AnimationPlayer, With<Armature>>,
    mut commands: Commands,
) {
    let Some(graph_handle) = &assets.graph else {
        warn!("Animation graph not ready yet");
        return;
    };
    let Some(node) = assets.node else {
        warn!("No animation node");
        return;
    };

    // Find the Armature entity (has AnimationPlayer)
    let mut armature_entity = None;
    for child in children.iter_descendants(trigger.entity) {
        if armature_query.get(child).is_ok() {
            armature_entity = Some(child);
            break;
        }
    }

    let Some(armature) = armature_entity else {
        warn!("No Armature with AnimationPlayer found");
        return;
    };

    // Attach animation graph
    commands
        .entity(armature)
        .insert(AnimationGraphHandle(graph_handle.clone()));

    // Play the animation
    if let Ok(mut player) = armature_query.get_mut(armature) {
        player.play(node).repeat();
        info!("Animation playing on human");
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

    paths.insert(node.name.clone(), path.clone());

    for child_handle in &node.children {
        build_gltf_paths(child_handle, &path, gltf_nodes, paths);
    }
}
