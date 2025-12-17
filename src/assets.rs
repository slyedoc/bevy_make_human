use crate::{components::*, loaders::*};
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use strum::{Display, EnumCount, EnumIter, EnumProperty, IntoEnumIterator};

// See build.rs for more details
// Asset enums generated at compile time
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Trait for assets with thumbnail
pub trait MHThumb: Copy + 'static {
    fn thumb(&self) -> &'static str;
}

/// Trait for MakeHuman part assets with mhclo/mhmat/obj/thumb
pub trait MHPart: MHThumb {
    fn mhclo(&self) -> &'static str;
    fn mhmat(&self) -> &'static str;
    fn obj(&self) -> &'static str;
}

/// Converts Enums to Handles
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HumanAssets {
    pub skin_obj_base: Handle<ObjBaseMesh>,
    pub skin_proxy: Handle<ProxyAsset>,
    pub skin_material: Handle<StandardMaterial>,

    pub parts: Vec<MHItem>,

    pub rig_bones: Handle<RigBones>,
    pub rig_weights: Handle<SkinningWeights>,

    /// All morph targets (body morphs + macro morphs)
    pub morphs: Vec<(Handle<MorphTargetData>, f32)>,
    /// Offset to push clothing outward (prevents skin poke-through)
    pub clothing_offset: f32,

    #[cfg(feature = "arkit")]
    /// ARKit blend shape targets (52 shapes)
    pub arkit_targets: Vec<Handle<MorphTargetData>>,
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

        #[cfg(feature = "arkit")]
        for handle in &self.arkit_targets {
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
    pub fn load<T: MHPart>(tag: MHTag, part: &T, asset_server: &AssetServer) -> Self {
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

/// A morph target with a value
/// - Binary morphs (body parts): -1.0 to 1.0 (neg=decr, pos=incr)
/// - Single/Macro morphs: 0.0 to 1.0
#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct Morph {
    pub target: MorphTarget,
    #[inspector(min = -1.0, max = 1.0, speed = 0.01, display = NumberDisplay::Slider)]
    pub value: f32,
}

impl Morph {
    pub fn new(target: MorphTarget, value: f32) -> Self {
        let (min, max) = target.value_range();
        Self {
            target,
            value: value.clamp(min, max),
        }
    }

    /// Create a macro morph (convenience for MorphTarget::Macro)
    pub fn macro_morph(target: MacroMorph, value: f32) -> Self {
        Self::new(MorphTarget::Macro(target), value)
    }
}

impl From<(MorphTarget, f32)> for Morph {
    fn from((target, value): (MorphTarget, f32)) -> Self {
        Self::new(target, value)
    }
}

// TODO: loading this a bit differently, assming file names by kebab-case, no using strum here
/// ARKit blend shapes (52 total) - A2F output order
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect,
)]
#[strum(serialize_all = "kebab-case")]
#[repr(usize)]
pub enum ARKit {
    // Eyes Left (0-6)
    EyeBlinkLeft = 0,
    EyeLookDownLeft = 1,
    EyeLookInLeft = 2,
    EyeLookOutLeft = 3,
    EyeLookUpLeft = 4,
    EyeSquintLeft = 5,
    EyeWideLeft = 6,

    // Eyes Right (7-13)
    EyeBlinkRight = 7,
    EyeLookDownRight = 8,
    EyeLookInRight = 9,
    EyeLookOutRight = 10,
    EyeLookUpRight = 11,
    EyeSquintRight = 12,
    EyeWideRight = 13,

    // Jaw (14-17)
    JawForward = 14,
    JawLeft = 15,
    JawRight = 16,
    JawOpen = 17,

    // Mouth (18-40)
    MouthClose = 18,
    MouthFunnel = 19,
    MouthPucker = 20,
    MouthLeft = 21,
    MouthRight = 22,
    MouthSmileLeft = 23,
    MouthSmileRight = 24,
    MouthFrownLeft = 25,
    MouthFrownRight = 26,
    MouthDimpleLeft = 27,
    MouthDimpleRight = 28,
    MouthStretchLeft = 29,
    MouthStretchRight = 30,
    MouthRollLower = 31,
    MouthRollUpper = 32,
    MouthShrugLower = 33,
    MouthShrugUpper = 34,
    MouthPressLeft = 35,
    MouthPressRight = 36,
    MouthLowerDownLeft = 37,
    MouthLowerDownRight = 38,
    MouthUpperUpLeft = 39,
    MouthUpperUpRight = 40,

    // Eyebrows (41-45)
    BrowDownLeft = 41,
    BrowDownRight = 42,
    BrowInnerUp = 43,
    BrowOuterUpLeft = 44,
    BrowOuterUpRight = 45,

    // Cheeks (46-48)
    CheekPuff = 46,
    CheekSquintLeft = 47,
    CheekSquintRight = 48,

    // Nose (49-50)
    NoseSneerLeft = 49,
    NoseSneerRight = 50,

    // Tongue (51)
    TongueOut = 51,
}

impl ARKit {
    /// Convert to index (0-51)
    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Convert from index (0-51)
    pub fn from_index(index: usize) -> Option<Self> {
        Self::iter().nth(index)
    }

    /// Get .target file path
    pub fn target_path(&self) -> String {
        format!("make_human/targets/arkit/{}.target", self)
    }
}
