use crate::{components::*, loaders::*};
use bevy::{prelude::*};
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};
#[allow(unused_imports)]
use strum::{Display, EnumCount, EnumIter, EnumProperty, IntoEnumIterator};

// See build.rs for more details
// Finding all the assets related to MakeHuman would be magic string hell
include!(concat!(env!("OUT_DIR"), "/assets.rs"));


#[derive(Component)]
pub struct HumanAssets {
    pub skin_obj_base: Handle<ObjBaseMesh>,
    pub skin_proxy: Handle<ProxyAsset>,
    pub skin_material: Handle<StandardMaterial>,
    
    pub parts: Vec<MHItem>,

    pub rig_bones: Handle<RigBones>,
    pub rig_weights: Handle<SkinningWeights>,

    pub morphs: Vec<(Handle<MorphTargetData>, f32)>,
    /// Phenotype macrodetail targets with interpolation weights
    pub phenotype_morphs: Vec<(Handle<MorphTargetData>, f32)>,
    /// Offset to push clothing outward (prevents skin poke-through)
    pub clothing_offset: f32,
}

/// Query params for building CharacterAssets from components
/// All part components are optional except Rig, Skin, Clothing
pub struct HumanComponents<'a> {
    pub rig: &'a Rig,
    pub skin: &'a Skin,
    pub eyes: Option<&'a Eyes>,
    pub eyebrows: Option<&'a Eyebrows>,
    pub eyelashes: Option<&'a Eyelashes>,
    pub teeth: Option<&'a Teeth>,
    pub tongue: Option<&'a Tongue>,
    pub hair: Option<&'a Hair>,
    pub clothing: &'a Clothing,
    pub morphs: &'a Morphs,
    pub phenotype: &'a Phenotype,
    pub clothing_offset: &'a ClothingOffset,
}

impl HumanAssets {
    /// Create from individual components
    pub fn from_components(
        c: HumanComponents,
        asset_server: &AssetServer,
    ) -> Self {
        let mut parts = Vec::new();

        load_part(c.eyes, &mut parts, asset_server);
        load_part(c.eyebrows, &mut parts, asset_server);
        load_part(c.eyelashes, &mut parts, asset_server);
        load_part(c.teeth, &mut parts, asset_server);
        load_part(c.tongue, &mut parts, asset_server);
        load_part(c.hair, &mut parts, asset_server);

        for clothing_item in c.clothing.0.iter() {
            parts.push(MHItem::load(
                MHTag::Clothes,
                clothing_item.mhclo().to_string(),
                clothing_item.mhmat().to_string(),
                clothing_item.obj().to_string(),
                asset_server,
            ));
        }

        Self {
            skin_obj_base: asset_server.load(c.skin.mesh.obj().to_string()),
            skin_proxy: asset_server.load(c.skin.mesh.proxy().to_string()),
            skin_material: asset_server.load(c.skin.material.mhmat().to_string()),

            rig_bones: asset_server.load(c.rig.rig_json_path().to_string()),
            rig_weights: asset_server.load(c.rig.weights().to_string()),            
            morphs: c.morphs.0
                .iter()
                .filter_map(|Morph { target, value }| {
                    target
                        .target_path(*value)
                        .map(|path| (asset_server.load(path.to_string()), *value))
                })
                .collect(),
            phenotype_morphs: c.phenotype
                .all_targets()
                .into_iter()
                .map(|(path, weight)| (asset_server.load(path), weight))
                .collect(),
            clothing_offset: c.clothing_offset.0,
            parts,
        }
    }

    /// Get all handles for progress tracking
    pub fn all_handles(&self) -> Vec<UntypedHandle> {
        let mut handles = vec![
            self.skin_obj_base.clone().untyped(),
            self.skin_proxy.clone().untyped(),
            self.skin_material.clone().untyped(),
            self.rig_bones.clone().untyped(),
            self.rig_weights.clone().untyped(),
        ];

        for part in &self.parts {
            handles.extend(part.handles());
        }

        for (handle, _value) in &self.morphs {
            handles.push(handle.clone().untyped());
        }

        for (handle, _weight) in &self.phenotype_morphs {
            handles.push(handle.clone().untyped());
        }

        handles
    }
}

// Helper to load optional MHPart components
fn load_part<T: MHPart>(
    part: Option<&T>,
    parts: &mut Vec<MHItem>,
    asset_server: &AssetServer,
) {
    if let Some(p) = part {
        parts.push(MHItem::load(
            T::tag(),
            p.mhclo().to_string(),
            p.mhmat().to_string(),
            p.obj().to_string(),
            asset_server,
        ));
    }
}




pub struct MHItem {
    pub tag: MHTag,
    pub clo: Handle<MhcloAsset>,
    pub mat: Handle<StandardMaterial>,
    pub obj_base: Handle<ObjBaseMesh>, // Mesh + original verts for mhid_lookup
}

impl MHItem {
    /// Load assets (clo, mat, obj with verts)
    pub fn load(
        tag: MHTag,
        clo_path: String,
        mat_path: String,
        obj_path: String,
        asset_server: &AssetServer,
    ) -> Self {
        Self {
            tag,
            clo: asset_server.load(clo_path),
            mat: asset_server.load(mat_path),
            obj_base: asset_server.load(obj_path),
        }
    }

    /// Get all handles for this item
    pub fn handles(&self) -> Vec<UntypedHandle> {
        vec![
            self.clo.clone().untyped(),
            self.mat.clone().untyped(),
            self.obj_base.clone().untyped(),
        ]
    }
}

pub struct MHItemLoaded {
    pub tag: MHTag,
    pub mat: Handle<StandardMaterial>, // dont do anything currently with material, but we need pass it along
    pub clo: MhcloAsset,
    pub base: ObjBaseMesh,
}

pub struct MHItemResult {
    pub tag: MHTag,
    pub mat: Handle<StandardMaterial>, // dont do anything currently with material, but we need pass it along
    pub mesh: Mesh,    
}


pub struct MHItemFinal {
    pub tag: MHTag,
    pub mat: Handle<StandardMaterial>, // dont do anything currently with material, but we need pass it along
    pub mesh: Handle<Mesh>,    
}


/// A morph target with a value from -1.0 to 1.0
/// Negative values use the "decr" target, positive use "incr"
#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Morph {
    pub target: MorphTarget,
    #[inspector(min = -1.0, max = 1.0, speed = 0.01, display = NumberDisplay::Slider)]
    pub value: f32,
}

impl Morph {
    pub fn new(target: MorphTarget, value: f32) -> Self {
        Self {
            target,
            value: value.clamp(-1.0, 1.0),
        }
    }
}

impl From<(MorphTarget, f32)> for Morph {
    fn from((target, value): (MorphTarget, f32)) -> Self {
        Self::new(target, value)
    }
}
