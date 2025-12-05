use bevy::prelude::*;
use crate::morphs::*;

/// Shape archetype - baked morph preset for runtime blending
/// These become Bevy mesh morph targets (up to 64 per mesh)
#[derive(Debug, Clone)]
pub struct ShapeArchetype {
    pub name: String,
    /// Baked morph values for this shape
    pub morphs: Vec<(MorphTarget, f32)>,
}

impl ShapeArchetype {
    pub fn new(name: impl Into<String>, morphs: Vec<(MorphTarget, f32)>) -> Self {
        Self {
            name: name.into(),
            morphs,
        }
    }
}

/// Component for runtime shape blending
#[derive(Component, Clone, Default)]
pub struct ShapeConfig {
    /// Shape blend weights (index matches shape archetype order)
    pub blend_weights: Vec<(usize, f32)>,
}

/// Helper to build mesh with morph targets from shape archetypes
pub fn build_shape_variants(
    base_mesh: &[Vec3],
    shapes: &[ShapeArchetype],
    morph_library: &MorphLibrary,
    morph_assets: &Assets<MorphTargetData>,
) -> Vec<Vec<Vec3>> {
    let mut variants = Vec::new();

    // Base variant (no morphs)
    variants.push(base_mesh.to_vec());

    // Each shape archetype becomes a morph target variant
    for shape in shapes {
        let morphed = apply_morphs(base_mesh, &shape.morphs, morph_library, morph_assets);
        variants.push(morphed);
    }

    variants
}

/// Apply shape blending via MorphWeights component
/// Note: MorphWeights requires mesh handle in Bevy 0.17
/// TODO: Integrate with mesh generation after GPU morph targets implemented
#[allow(dead_code)]
pub fn apply_shape_weights(
    _config: &ShapeConfig,
    _parent_entity: Entity,
    _mesh_handle: Handle<Mesh>,
    _commands: &mut Commands,
) {
    // TODO: MorphWeights::new(weights, Some(mesh_handle))
    // Requires mesh with morph targets set up
}

/// Example predefined shape archetypes
/// NOTE: Use actual generated enum variants from target.json
pub mod presets {
    use super::*;
    use crate::*;

    pub fn muscular() -> ShapeArchetype {
        ShapeArchetype::new(
            "Muscular",
            vec![
                (MorphTarget::Torso(TorsoMorph::TorsoMusclePectoralIncr), 0.5),
                (MorphTarget::Arms(ArmsMorph::LUpperarmFatDecr), 0.3),
                (MorphTarget::Arms(ArmsMorph::RUpperarmFatDecr), 0.3),
            ],
        )
    }

    pub fn slender() -> ShapeArchetype {
        ShapeArchetype::new(
            "Slender",
            vec![
                (MorphTarget::Torso(TorsoMorph::TorsoScaleHorizDecr), 0.4),
                (MorphTarget::Arms(ArmsMorph::MeasureUpperarmCircDecr), 0.3),
            ],
        )
    }
}
