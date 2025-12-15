
#[path = "common/mod.rs"]
mod common;
pub use common::*;

use avian3d::prelude::*;
use bevy::{mesh::morph::MeshMorphWeights, prelude::*};
use bevy_make_human::prelude::*;
use strum::EnumCount;

#[derive(Component)]
struct TestAnimation {
    time: f32,
    shape: ARKit,
}

fn main() -> AppExit {
    
    // WIP testing animating ARKit blend shapes
    // currnelty reusing custom targets mapped to ARKit shapes, 
    // TODO: learn to create proper ARKit blend shapes for base model    
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
            CommonPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, test_animation)
        .add_observer(on_human_complete)
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
        Transform::from_xyz(0.0, 1.6, 1.0).looking_at(Vec3::new(0.0, 1.4, 0.0), Vec3::Y),
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
        RigidBody::Static,
        Collider::half_space(Vec3::Y),
        Transform::default(),
    ));

    commands.spawn((
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
        Morphs(vec![Morph::new(
            MorphTarget::Macro(MacroMorph::CaucasianMaleYoung),
            1.0,
        )]),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    info!("Press 1-8 for specific shapes, SPACE to cycle");
}

fn on_human_complete(trigger: On<HumanComplete>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(TestAnimation {
        time: 0.0,
        shape: ARKit::JawOpen,
    });
    info!("ARKit morphs ready ({} shapes)", ARKit::COUNT);
}

fn test_animation(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut MeshMorphWeights, &mut TestAnimation)>,
) {
    for (mut weights, mut anim) in query.iter_mut() {
        let w = weights.weights_mut();

        // Reset all
        w.fill(0.0);

        // Key controls
        if keyboard.just_pressed(KeyCode::Digit1) {
            anim.shape = ARKit::JawOpen;
        }
        if keyboard.just_pressed(KeyCode::Digit2) {
            anim.shape = ARKit::MouthSmileLeft;
        }
        if keyboard.just_pressed(KeyCode::Digit3) {
            anim.shape = ARKit::MouthSmileRight;
        }
        if keyboard.just_pressed(KeyCode::Digit4) {
            anim.shape = ARKit::EyeBlinkLeft;
        }
        if keyboard.just_pressed(KeyCode::Digit5) {
            anim.shape = ARKit::EyeBlinkRight;
        }
        if keyboard.just_pressed(KeyCode::Digit6) {
            anim.shape = ARKit::BrowInnerUp;
        }
        if keyboard.just_pressed(KeyCode::Digit7) {
            anim.shape = ARKit::CheekPuff;
        }
        if keyboard.just_pressed(KeyCode::Digit8) {
            anim.shape = ARKit::TongueOut;
        }

        // Cycle with space
        if keyboard.just_pressed(KeyCode::Space) {
            let next = (anim.shape.as_index() + 1) % ARKit::COUNT;
            anim.shape = ARKit::from_index(next).unwrap();
            info!("Shape {}: {}", next, anim.shape);
        }

        // Animate
        anim.time += time.delta_secs();
        let value = (anim.time * 2.0).sin() * 0.5 + 0.5;
        w[anim.shape.as_index()] = value;
    }
}
