use crate::assets::*;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;

/// Main Human marker - only truly required components
/// Eyes, Eyebrows, Eyelashes, Teeth, Tongue, Hair are all optional
#[derive(Component, Default)]
#[require(
    Phenotype,
    Transform,
    Visibility,
    Rig,
    Skin,
    Eyes,
    Eyebrows,
    Eyelashes,
    Clothing,
    ClothingOffset,
    FloorOffset,
    Morphs
)]
pub struct Human;

#[derive(Component, Clone, Reflect)]
pub struct Skin {
    pub mesh: SkinMesh,
    pub material: SkinMaterial,
}

impl Default for Skin {
    fn default() -> Self {
        Self {
            mesh: SkinMesh::FemaleGeneric,
            material: SkinMaterial::YoungCaucasianFemale,
        }
    }
}

// === PARTS ===
// Eyes, Eyebrows, Eyelashes, Teeth, Tongue, Hair are generated in build.rs with Component derive
// They implement MHPart trait for generic handling

// Clothing is the only multi-item part, needs wrapper
#[derive(Component, Clone, Default, Debug, Reflect, Deref, DerefMut)]
pub struct Clothing(pub Vec<ClothingAsset>);

/// Uses normals to offset clothing away from skin, hack to reduce z-fighting
// TODO: replace with Delete Vertex Groups
#[derive(Component, Clone, Copy, Default, Debug, Reflect, InspectorOptions, Deref, DerefMut)]
#[reflect(Component, InspectorOptions)]
pub struct ClothingOffset(
    #[inspector(min = 0.0, max = 0.01, speed = 0.0001, display = NumberDisplay::Slider)] pub f32,
);

/// Vertical offset to adjust for floor contact (shoes, bare feet, etc)
#[derive(Component, Clone, Copy, Default, Debug, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct FloorOffset(
    #[inspector(min = -0.1, max = 0.1, speed = 0.001, display = NumberDisplay::Slider)] pub f32,
);

#[derive(Component, Clone, Debug, Default)]
pub struct Morphs(pub Vec<Morph>);

// Marker components body parts
#[derive(Component, Copy, Clone, strum::Display, PartialEq, Eq, Debug, Reflect)]
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
    Collider,
}

#[derive(Component, Default)]
pub struct Armature;

#[derive(Component)]
#[require(MHTag::Skin)]
pub struct SkinPart;

#[derive(Component)]
#[require(MHTag::Hair)]
pub struct HairPart;

#[derive(Component)]
#[require(MHTag::Eyes)]
pub struct EyesPart;

#[derive(Component)]
#[require(MHTag::Teeth)]
pub struct TeethPart;

#[derive(Component)]
#[require(MHTag::Tongue)]
pub struct TonguePart;

#[derive(Component)]
#[require(MHTag::Eyebrows)]
pub struct EyebrowsPart;

#[derive(Component)]
#[require(MHTag::Eyelashes)]
pub struct EyelashesPart;

#[derive(Component)]
#[require(MHTag::Clothes)]
pub struct ClothesPart;

// TODO: Below Here, only used in raven_ai for now

#[derive(Component)]
pub struct LeftEyeMesh;

#[derive(Component)]
pub struct RightEyeMesh;

#[derive(Component)]
pub struct JawMesh;

#[derive(Component)]
pub struct UpperJawMesh;
