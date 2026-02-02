//! Animation example using UAL1 animations (tower approach)
//!
//! This example uses the same animation loading approach as the tower game,
//! loading animations directly from UAL1_Standard.glb without curve filtering.

#[path = "common/mod.rs"]
mod common;
use common::*;

use avian3d::prelude::*;
use bevy::{
    animation::{AnimatedBy, AnimationTargetId},
    gltf::Gltf,
    prelude::*,
};
use bevy_make_human::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
            CommonPlugin,
        ))
        .init_resource::<AnimationAssets>()
        .add_systems(Startup, (load_animations, setup))
        .add_systems(Update, check_animations_loaded)
        .run()
}

#[derive(Resource, Default)]
struct AnimationAssets {
    gltf: Handle<Gltf>,
    graph: Option<Handle<AnimationGraph>>,
    idle: Option<AnimationNodeIndex>,
    run: Option<AnimationNodeIndex>,
    jump: Option<AnimationNodeIndex>,
}

fn load_animations(mut assets: ResMut<AnimationAssets>, asset_server: Res<AssetServer>) {
    // Load UAL1 animations (same as tower game)
    assets.gltf = asset_server.load("animations/UAL1_Standard.glb");
}

fn check_animations_loaded(
    mut assets: ResMut<AnimationAssets>,
    gltfs: Res<Assets<Gltf>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Only build once
    if assets.graph.is_some() {
        return;
    }

    let Some(gltf) = gltfs.get(&assets.gltf) else {
        return;
    };

    info!("Building animation graph from UAL1");

    // Build animation graph directly - no curve filtering needed
    let mut graph = AnimationGraph::new();

    let idle = graph.add_clip(
        gltf.named_animations["Idle_Loop"].clone(),
        1.0,
        graph.root,
    );
    let run = graph.add_clip(
        gltf.named_animations["Jog_Fwd_Loop"].clone(),
        1.0,
        graph.root,
    );
    let jump = graph.add_clip(
        gltf.named_animations["Jump_Loop"].clone(),
        1.0,
        graph.root,
    );

    assets.graph = Some(animation_graphs.add(graph));
    assets.idle = Some(idle);
    assets.run = Some(run);
    assets.jump = Some(jump);

    info!("Animation graph ready with Idle, Run, Jump");
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

    // Spawn human with UAL1-compatible rig
    commands
        .spawn((
            Name::new("AnimatedHuman"),
            Human,
            Rig::Ual1, // UAL1-compatible rig with matching bone names
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
            Transform::IDENTITY, // No rotation needed - both use Y-up
        ))
        .observe(on_human_complete);
}

/// Called when the human is fully generated
fn on_human_complete(
    trigger: On<HumanComplete>,
    assets: Res<AnimationAssets>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    armature_query: Query<Entity, With<Armature>>,
    mut commands: Commands,
) {
    let Some(graph_handle) = &assets.graph else {
        warn!("Animation graph not ready yet");
        return;
    };
    let Some(idle_node) = assets.idle else {
        warn!("No idle animation node");
        return;
    };

    // Find the Armature entity
    let mut armature_entity = None;
    for child in children.iter_descendants(trigger.entity) {
        if armature_query.get(child).is_ok() {
            armature_entity = Some(child);
            break;
        }
    }

    let Some(armature) = armature_entity else {
        warn!("No Armature found");
        return;
    };

    info!("Setting up animation retargeting (tower approach)");

    // Get armature name for path prefix
    let armature_name = Name::new("Armature");

    // Add AnimationTarget to Armature itself
    let armature_path = vec![armature_name.clone()];
    let armature_target_id = AnimationTargetId::from_names(armature_path.iter());
    commands
        .entity(armature)
        .insert((armature_target_id, AnimatedBy(armature)));

    // Add AnimationTarget to all bones under Armature
    let mut target_count = 1;

    for descendant in children.iter_descendants(armature) {
        // Build path from this bone up to (but not including) armature
        let mut path: Vec<Name> = Vec::new();
        let mut current = descendant;

        while current != armature {
            if let Ok(name) = names.get(current) {
                path.push(name.clone());
            }
            if let Ok(parent) = parents.get(current) {
                current = parent.parent();
            } else {
                break;
            }
        }

        path.reverse();

        // Prepend armature name - GLTF animation paths include it
        let mut full_path = vec![armature_name.clone()];
        full_path.extend(path);

        let target_id = AnimationTargetId::from_names(full_path.iter());
        target_count += 1;
        commands
            .entity(descendant)
            .insert((target_id, AnimatedBy(armature)));
    }
    info!("Added {} animation targets", target_count);

    // Add AnimationPlayer to Armature
    let mut player = AnimationPlayer::default();
    player.play(idle_node).repeat();
    commands.entity(armature).insert((
        player,
        AnimationGraphHandle(graph_handle.clone()),
        AnimationTransitions::default(),
    ));

    info!("Animation playing on human");
}
