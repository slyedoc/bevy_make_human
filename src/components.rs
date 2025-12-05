use crate::assets::*;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;

#[derive(Component, Default)]
#[require(
    Phenotype,
    Transform,
    Visibility,
    Rig,
    Skin,
    Clothing,
    ClothingOffset,
    FloorOffset
)]
pub struct Human;

#[derive(Component, Clone)]
pub struct Skin {
    /// Optional proxy mesh. If None, uses base mesh.
    pub mesh: Option<SkinMesh>,
    pub material: SkinMaterial,
}

impl Default for Skin {
    fn default() -> Self {
        Self {
            mesh: None,
            material: SkinMaterial::YoungCaucasianFemale,
        }
    }
}

// === PARTS ===

#[derive(Component, Clone, Copy)]
pub struct Eyes {
    pub mesh: EyesMesh,
    pub material: EyesMaterial,
}

// Hair is generated in build.rs with Component derive

#[derive(Component, Clone, Copy)]
pub struct Eyebrows(pub EyebrowsAsset);

#[derive(Component, Clone, Copy)]
pub struct Eyelashes(pub EyelashesAsset);

#[derive(Component, Clone, Copy)]
pub struct Teeth(pub TeethAsset);

#[derive(Component, Clone, Copy)]
pub struct Tongue(pub TongueAsset);

#[derive(Component, Clone, Default)]
pub struct Clothing(pub Vec<ClothingAsset>);

// === SETTINGS ===

#[derive(Component, Clone, Copy, Default, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct ClothingOffset(
    #[inspector(min = 0.0, max = 0.01, speed = 0.0001, display = NumberDisplay::Slider)] pub f32,
);

#[derive(Component, Clone, Copy, Default, Reflect, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct FloorOffset(
    #[inspector(min = -0.1, max = 0.1, speed = 0.001, display = NumberDisplay::Slider)] pub f32,
);

#[derive(Component, Clone, Default)]
pub struct Morphs(pub Vec<Morph>);

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
    Collider,
}

#[derive(Component, Default)]
pub struct HumanPart;

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
