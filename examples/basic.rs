#[path ="common/mod.rs"]
mod common;
pub use common::*;

use bevy::prelude::*;
use avian3d::prelude::*;
use bevy_make_human::prelude::*;
use bevy_inspector_egui::quick::StateInspectorPlugin;
fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // for faster load times, requires: "bevy/asset_processor",
            // .set(AssetPlugin {  
            //     mode: AssetMode::Processed,
            //     ..default()
            // }),

            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
            CommonPlugin, // camera and egui editor
            StateInspectorPlugin::<MHState>::default(),
        ))
        .add_systems(Startup, setup)
        .run()
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

    commands.spawn((
        Name::new("Bob"),
        Human,
        Rig::Mixamo,
        Skin {
            mesh: Some(SkinMesh::MaleGeneric),
            material: SkinMaterial::YoungCaucasianMale,
        },
        Eyes {
            mesh: EyesMesh::LowPoly,
            material: EyesMaterial::Bluegreen,
        },
        Hair::Bob02,
        Eyebrows(EyebrowsAsset::Eyebrow006),
        Eyelashes(EyelashesAsset::Eyelashes01),
        Teeth(TeethAsset::TeethBase),
        Tongue(TongueAsset::Tongue01),
        Clothing(vec![
            ClothingAsset::ToigoMaleSuit3,
            ClothingAsset::ToigoAnkleBootsMale,
        ]),
        Morphs::default(),
        Phenotype {
            gender: 1.0,
            age: 0.5,
            muscle: 0.3,
            weight: 0.4,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
