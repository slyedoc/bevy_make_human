use crate::{components::*, loaders::*};
use bevy::{prelude::*};
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};
#[allow(unused_imports)]
use strum::{Display, EnumCount, EnumIter, EnumProperty, IntoEnumIterator};

/// Trait for MakeHuman part assets with mhclo/mhmat/obj/thumb
pub trait MHPart: Copy + 'static {
    fn mhclo(&self) -> &str;
    fn mhmat(&self) -> &str;
    fn obj(&self) -> &str;
    fn thumb(&self) -> &str;
}


// See build.rs for more details
// Finding all the assets related to MakeHuman would be magic string hell
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Converts Enums to Handles
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

impl HumanAssets {
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

pub struct MHItem {
    pub tag: MHTag,
    pub clo: Handle<MhcloAsset>,
    pub mat: Handle<StandardMaterial>,
    pub obj_base: Handle<ObjBaseMesh>, // Mesh + original verts for mhid_lookup
}

impl MHItem {
    /// Load assets (clo, mat, obj with verts)
    pub fn load<T: MHPart>(tag: MHTag, part: &T, asset_server: &AssetServer,
    ) -> Self {
        Self {
            tag,
            clo: asset_server.load(part.mhclo().to_string()),
            mat: asset_server.load(part.mhmat().to_string()),
            obj_base: asset_server.load(part.obj().to_string()),
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
