use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_make_human::prelude::*;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            MakeHumanPlugin::default(),
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
        Transform::from_xyz(-1.0, 0.0, 0.0),
    ));

    commands.spawn((
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
        Outfit(vec![Clothing::ElvsGoddessDress8]),
        Morphs(vec![Morph::new(
            MorphTarget::Macro(MacroMorph::CaucasianFemaleYoung),
            1.0,
        )]),
        Transform::from_xyz(1.0, 0.0, 0.0),
    ));
}
