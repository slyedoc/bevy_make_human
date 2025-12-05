use bevy::prelude::*;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use crate::assets::*;

#[derive(Component, Default)]
#[require(HumanConfig, Phenotype, Transform, Visibility)]
pub struct Human;

// TODO: break into componets?
#[derive(Component, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct HumanConfig {
    pub proxy_mesh: ProxyMesh,
    pub rig: RigAsset,
    pub skin: SkinAsset,
    pub hair: Option<HairAsset>,
    pub eyes: EyesAsset,
    pub eye_material: EyeMaterialAsset,
    pub eyebrows: EyebrowsAsset,
    pub eyelashes: EyelashesAsset,
    pub teeth: TeethAsset,
    pub tongue: TongueAsset,
    pub clothing: Vec<ClothingAsset>,
    pub morphs: Vec<Morph>,
    /// Offset to push clothing outward along surface normal (prevents skin poke-through)
    #[inspector(min = 0.0, max = 0.01, speed = 0.0001, display = NumberDisplay::Slider)]
    pub clothing_offset: f32,
    /// Floor offset for collider (accounts for shoes/bare feet)
    #[inspector(min = -0.1, max = 0.1, speed = 0.001, display = NumberDisplay::Slider)]
    pub floor_offset: f32,
}

impl Default for HumanConfig {
    fn default() -> Self {
        Self {
            proxy_mesh: ProxyMesh::FemaleGeneric,
            rig: RigAsset::Default, // CMU Motion Builder rig - maps to SMPL joints
            skin: SkinAsset::YoungCaucasianFemale,
            hair: Some(HairAsset::GrinsegoldWigBowTie),
            eyes: EyesAsset::LowPoly,
            eyelashes: EyelashesAsset::Eyelashes01,
            teeth: TeethAsset::TeethBase,
            tongue: TongueAsset::Tongue01,
            eye_material: EyeMaterialAsset::Brown,
            eyebrows: EyebrowsAsset::Eyebrow006,
            clothing: vec![ClothingAsset::ElvsSarongCoverUp],
            morphs: vec![],
            clothing_offset: 0.001,
            floor_offset: 0.0,
        }
    }
}

// Marker components body parts
#[derive(Component, Copy, Clone, strum::Display, PartialEq, Eq, Debug)]
pub enum MHTag {
    Armature,
    Skin,
    Hair,
    Eyes,
    Teeth,
    Tongue,
    Eyebrows,
    Eyelashes,
    Clothes,
    Collider
}

#[derive(Component, Default)]
pub struct HumanPart;

#[derive(Component)]
#[require(MHTag::Skin)]
pub struct SkinMesh;

#[derive(Component)]
#[require(MHTag::Hair)]
pub struct HairMesh;

#[derive(Component)]
#[require(MHTag::Eyes)]
pub struct EyesMesh;

#[derive(Component)]
#[require(MHTag::Teeth)]
pub struct TeethMesh;

#[derive(Component)]
#[require(MHTag::Tongue)]
pub struct TongueMesh;

#[derive(Component)]
#[require(MHTag::Eyebrows)]
pub struct EyebrowsMesh;

#[derive(Component)]
#[require(MHTag::Eyelashes)]
pub struct EyelashesMesh;

#[derive(Component)]
#[require(MHTag::Clothes)]
pub struct ClothesMesh;


// TODO: Below Here, only used in raven_ai for now

#[derive(Component)]
pub struct LeftEyeMesh;

#[derive(Component)]
pub struct RightEyeMesh;

#[derive(Component)]
pub struct JawMesh;

#[derive(Component)]
pub struct UpperJawMesh;