use crate::{components::*, loaders::*};
use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::std_options::NumberDisplay, prelude::*};
#[allow(unused_imports)]
use strum::{Display, EnumCount, EnumIter, EnumProperty, IntoEnumIterator};

// See build.rs for more details
// Finding all the assets related to MakeHuman would be magic string hell
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

#[derive(Component)]
pub struct CharacterAssets {
    /// Proxy mesh obj+verts (None = use base mesh)
    pub skin_obj_base: Option<Handle<ObjBaseMesh>>,
    pub skin_proxy: Option<Handle<ProxyAsset>>,
    pub skin_material: Handle<StandardMaterial>,
    pub parts: Vec<MHItem>,

    pub rig_bones: Handle<RigBones>,
    pub rig_weights: Handle<SkinningWeights>,
    /// Optional skeleton GLB for base rotations (animation compatibility)
    pub skeleton_glb: Option<Handle<Gltf>>,

    pub morphs: Vec<(Handle<MorphTargetData>, f32)>,
    /// Phenotype macrodetail targets with interpolation weights
    pub phenotype_morphs: Vec<(Handle<MorphTargetData>, f32)>,
    /// Offset to push clothing outward (prevents skin poke-through)
    pub clothing_offset: f32,
}

/// Query params for building CharacterAssets from components
pub struct CharacterComponents<'a> {
    pub rig: &'a Rig,
    pub skin: &'a Skin,
    pub eyes: &'a Eyes,
    pub eyebrows: &'a Eyebrows,
    pub eyelashes: &'a Eyelashes,
    pub teeth: &'a Teeth,
    pub tongue: &'a Tongue,
    pub hair: Option<&'a Hair>,
    pub clothing: &'a Clothing,
    pub morphs: &'a Morphs,
    pub phenotype: &'a Phenotype,
    pub clothing_offset: f32,
}

impl CharacterAssets {
    /// Create from individual components
    pub fn from_components(
        c: CharacterComponents,
        asset_server: &AssetServer,
    ) -> Self {
        let mut parts = vec![
            MHItem::load(
                MHTag::Eyes,
                c.eyes.mesh.mhclo_path().to_string(),
                c.eyes.material.mhmat_path().to_string(),
                c.eyes.mesh.obj_path().to_string(),
                asset_server,
            ),
            MHItem::load(
                MHTag::Teeth,
                c.teeth.0.mhclo_path().to_string(),
                c.teeth.0.mhmat_path().to_string(),
                c.teeth.0.obj_path().to_string(),
                asset_server,
            ),
            MHItem::load(
                MHTag::Tongue,
                c.tongue.0.mhclo_path().to_string(),
                c.tongue.0.mhmat_path().to_string(),
                c.tongue.0.obj_path().to_string(),
                asset_server,
            ),
            MHItem::load(
                MHTag::Eyebrows,
                c.eyebrows.0.mhclo_path().to_string(),
                c.eyebrows.0.mhmat_path().to_string(),
                c.eyebrows.0.obj_path().to_string(),
                asset_server,
            ),
            MHItem::load(
                MHTag::Eyelashes,
                c.eyelashes.0.mhclo_path().to_string(),
                c.eyelashes.0.mhmat_path().to_string(),
                c.eyelashes.0.obj_path().to_string(),
                asset_server,
            ),
        ];

        if let Some(hair) = c.hair {
            parts.push(MHItem::load(
                MHTag::Hair,
                hair.mhclo_path().to_string(),
                hair.mhmat_path().to_string(),
                hair.obj_path().to_string(),
                asset_server,
            ));
        }

        for clothing_item in c.clothing.0.iter() {
            parts.push(MHItem::load(
                MHTag::Clothes,
                clothing_item.mhclo_path().to_string(),
                clothing_item.mhmat_path().to_string(),
                clothing_item.obj_path().to_string(),
                asset_server,
            ));
        }

        Self {
            skin_obj_base: c.skin.mesh.as_ref().map(|m| asset_server.load(m.obj_path().to_string())),
            skin_proxy: c.skin.mesh.as_ref().map(|m| asset_server.load(m.proxy_path().to_string())),
            skin_material: asset_server.load(c.skin.material.mhmat_path().to_string()),

            rig_bones: asset_server.load(c.rig.rig_json_path().to_string()),
            rig_weights: asset_server.load(c.rig.weights_json_path().to_string()),
            skeleton_glb: c.rig
                .skeleton_glb_path()
                .map(|p| asset_server.load(p.to_string())),
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
            clothing_offset: c.clothing_offset,
            parts,
        }
    }

    /// Get all handles for progress tracking
    pub fn all_handles(&self) -> Vec<UntypedHandle> {
        let mut handles = vec![
            self.skin_material.clone().untyped(),
            self.rig_bones.clone().untyped(),
            self.rig_weights.clone().untyped(),
        ];

        if let Some(ref h) = self.skin_obj_base {
            handles.push(h.clone().untyped());
        }
        if let Some(ref h) = self.skin_proxy {
            handles.push(h.clone().untyped());
        }

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

/// Trait to get thumbnail path from an enum variant
pub trait HasThumbnail {
    fn thumbnail_path(&self) -> Option<&'static str>;
}

impl HasThumbnail for SkinMesh {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for SkinMaterial {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for Hair {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for EyesMesh {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for EyebrowsAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for EyelashesAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for TeethAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for TongueAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for Rig {
    fn thumbnail_path(&self) -> Option<&'static str> {
        None
    } // No thumbs for rigs
}
impl HasThumbnail for EyesMaterial {
    fn thumbnail_path(&self) -> Option<&'static str> {
        None
    } // No thumbs for eye materials
}
impl HasThumbnail for ClothingAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl HasThumbnail for PoseAsset {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.get_str("thumb")
    }
}
impl<T: HasThumbnail> HasThumbnail for Option<T> {
    fn thumbnail_path(&self) -> Option<&'static str> {
        self.as_ref().and_then(|v| v.thumbnail_path())
    }
}
