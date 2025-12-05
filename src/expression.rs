// // WIP: trying to convert ARKit blend shapes to MakeHuman faceshapes
// //! ARKit to MakeHuman faceshapes mapping
// //!
// //! Maps 52 ARKit FaceShape blend weights to MakeHuman faceshapes (.mxa format).
// //! The faceshapes.mxa file contains FACS-based expressions that map much better
// //! to ARKit than the original MH expression targets.

// use bevy::{mesh::VertexAttributeValues, prelude::*};
// use crate::loaders::FaceshapesData;

// /// ARKit FaceShape index (0-51) to MH faceshape mapping
// #[derive(Debug, Clone)]
// pub struct FaceshapeMapping {
//     /// MH faceshape name(s) from faceshapes.mxa
//     /// Some ARKit shapes map to multiple MH shapes (e.g., smile = corner_up + wide)
//     pub shapes: &'static [(&'static str, f32)], // (name, scale)
//     /// Whether this should be handled by bone transform instead/additionally
//     pub use_bone: bool,
//     /// Bone name if use_bone is true
//     pub bone: Option<&'static str>,
// }

// impl Default for FaceshapeMapping {
//     fn default() -> Self {
//         Self {
//             shapes: &[],
//             use_bone: false,
//             bone: None,
//         }
//     }
// }

// /// Get the faceshape mapping for all 52 ARKit shapes
// /// Returns array indexed by FaceShape enum value (0-51)
// pub fn arkit_to_faceshapes() -> [FaceshapeMapping; 52] {
//     [
//         // 0: EyeBlinkLeft - use combined blink shape
//         FaceshapeMapping { shapes: &[("blink", 0.5)], ..default() },
//         // 1: EyeLookDownLeft - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.L"), ..default() },
//         // 2: EyeLookInLeft - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.L"), ..default() },
//         // 3: EyeLookOutLeft - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.L"), ..default() },
//         // 4: EyeLookUpLeft - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.L"), ..default() },
//         // 5: EyeSquintLeft - cheek squint affects eye area
//         FaceshapeMapping { shapes: &[("cheek_squint_left", 1.0)], ..default() },
//         // 6: EyeWideLeft - no direct equivalent, could use negative blink
//         FaceshapeMapping::default(),

//         // 7: EyeBlinkRight
//         FaceshapeMapping { shapes: &[("blink", 0.5)], ..default() },
//         // 8: EyeLookDownRight - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.R"), ..default() },
//         // 9: EyeLookInRight - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.R"), ..default() },
//         // 10: EyeLookOutRight - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.R"), ..default() },
//         // 11: EyeLookUpRight - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("eye.R"), ..default() },
//         // 12: EyeSquintRight
//         FaceshapeMapping { shapes: &[("cheek_squint_right", 1.0)], ..default() },
//         // 13: EyeWideRight
//         FaceshapeMapping::default(),

//         // 14: JawForward - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("jaw"), ..default() },
//         // 15: JawLeft - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("jaw"), ..default() },
//         // 16: JawRight - use bone
//         FaceshapeMapping { use_bone: true, bone: Some("jaw"), ..default() },
//         // 17: JawOpen
//         FaceshapeMapping { shapes: &[("mouth_open", 1.0)], use_bone: true, bone: Some("jaw") },

//         // 18: MouthClose - lips together
//         FaceshapeMapping { shapes: &[("lips_part", -0.5)], ..default() },
//         // 19: MouthFunnel - O shape, narrow + out
//         FaceshapeMapping { shapes: &[("mouth_narrow_left", 0.5), ("mouth_narrow_right", 0.5), ("lips_lower_out", 0.3), ("lips_upper_out", 0.3)], ..default() },
//         // 20: MouthPucker - kiss shape
//         FaceshapeMapping { shapes: &[("mouth_narrow_left", 1.0), ("mouth_narrow_right", 1.0)], ..default() },
//         // 21: MouthLeft - shift mouth left
//         FaceshapeMapping { shapes: &[("mouth_up_left", 0.3), ("mouth_down_left", 0.3)], ..default() },
//         // 22: MouthRight - shift mouth right
//         FaceshapeMapping { shapes: &[("mouth_up_right", 0.3), ("mouth_down_right", 0.3)], ..default() },
//         // 23: MouthSmileLeft
//         FaceshapeMapping { shapes: &[("mouth_corner_up_left", 1.0), ("mouth_wide_left", 0.3)], ..default() },
//         // 24: MouthSmileRight
//         FaceshapeMapping { shapes: &[("mouth_corner_up_right", 1.0), ("mouth_wide_right", 0.3)], ..default() },
//         // 25: MouthFrownLeft
//         FaceshapeMapping { shapes: &[("mouth_corner_down_left", 1.0)], ..default() },
//         // 26: MouthFrownRight
//         FaceshapeMapping { shapes: &[("mouth_corner_down_right", 1.0)], ..default() },
//         // 27: MouthDimpleLeft - corner in
//         FaceshapeMapping { shapes: &[("mouth_corner_in_left", 1.0)], ..default() },
//         // 28: MouthDimpleRight
//         FaceshapeMapping { shapes: &[("mouth_corner_in_right", 1.0)], ..default() },
//         // 29: MouthStretchLeft - wide mouth
//         FaceshapeMapping { shapes: &[("mouth_wide_left", 1.0)], ..default() },
//         // 30: MouthStretchRight
//         FaceshapeMapping { shapes: &[("mouth_wide_right", 1.0)], ..default() },
//         // 31: MouthRollLower - lip in
//         FaceshapeMapping { shapes: &[("lips_lower_in", 1.0)], ..default() },
//         // 32: MouthRollUpper
//         FaceshapeMapping { shapes: &[("lips_upper_in", 1.0)], ..default() },
//         // 33: MouthShrugLower - lip out
//         FaceshapeMapping { shapes: &[("lips_lower_out", 1.0)], ..default() },
//         // 34: MouthShrugUpper
//         FaceshapeMapping { shapes: &[("lips_upper_out", 1.0)], ..default() },
//         // 35: MouthPressLeft - press lips
//         FaceshapeMapping { shapes: &[("lips_mid_lower_up_left", 0.5), ("lips_mid_upper_down_left", 0.5)], ..default() },
//         // 36: MouthPressRight
//         FaceshapeMapping { shapes: &[("lips_mid_lower_up_right", 0.5), ("lips_mid_upper_down_right", 0.5)], ..default() },
//         // 37: MouthLowerDownLeft
//         FaceshapeMapping { shapes: &[("lips_mid_lower_down_left", 1.0)], ..default() },
//         // 38: MouthLowerDownRight
//         FaceshapeMapping { shapes: &[("lips_mid_lower_down_right", 1.0)], ..default() },
//         // 39: MouthUpperUpLeft
//         FaceshapeMapping { shapes: &[("lips_mid_upper_up_left", 1.0)], ..default() },
//         // 40: MouthUpperUpRight
//         FaceshapeMapping { shapes: &[("lips_mid_upper_up_right", 1.0)], ..default() },

//         // 41: BrowDownLeft
//         FaceshapeMapping { shapes: &[("brow_mid_down_left", 0.7), ("brow_outer_down_left", 0.3)], ..default() },
//         // 42: BrowDownRight
//         FaceshapeMapping { shapes: &[("brow_mid_down_right", 0.7), ("brow_outer_down_right", 0.3)], ..default() },
//         // 43: BrowInnerUp - both inner brows + squeeze
//         FaceshapeMapping { shapes: &[("brow_mid_up_left", 0.5), ("brow_mid_up_right", 0.5), ("brow_squeeze", 0.3)], ..default() },
//         // 44: BrowOuterUpLeft
//         FaceshapeMapping { shapes: &[("brow_outer_up_left", 1.0)], ..default() },
//         // 45: BrowOuterUpRight
//         FaceshapeMapping { shapes: &[("brow_outer_up_right", 1.0)], ..default() },

//         // 46: CheekPuff
//         FaceshapeMapping { shapes: &[("cheek_balloon_left", 1.0), ("cheek_balloon_right", 1.0)], ..default() },
//         // 47: CheekSquintLeft
//         FaceshapeMapping { shapes: &[("cheek_squint_left", 1.0), ("cheek_up_left", 0.5)], ..default() },
//         // 48: CheekSquintRight
//         FaceshapeMapping { shapes: &[("cheek_squint_right", 1.0), ("cheek_up_right", 0.5)], ..default() },

//         // 49: NoseSneerLeft
//         FaceshapeMapping { shapes: &[("nose_wrinkle", 0.5)], ..default() },
//         // 50: NoseSneerRight
//         FaceshapeMapping { shapes: &[("nose_wrinkle", 0.5)], ..default() },

//         // 51: TongueOut
//         FaceshapeMapping { shapes: &[("tongue_out", 1.0)], use_bone: true, bone: Some("tongue_base") },
//     ]
// }

// fn default() -> FaceshapeMapping {
//     FaceshapeMapping::default()
// }

// /// Component for expression weights on a character
// /// Stores 52 ARKit blend shape weights that get mapped to MH faceshapes
// #[derive(Component, Clone, Debug, Reflect)]
// pub struct ExpressionWeights {
//     /// 52 ARKit blend shape weights [0.0-1.0]
//     pub weights: [f32; 52],
// }

// impl Default for ExpressionWeights {
//     fn default() -> Self {
//         Self { weights: [0.0; 52] }
//     }
// }

// impl ExpressionWeights {
//     pub fn set_weight(&mut self, index: usize, value: f32) {
//         if index < 52 {
//             self.weights[index] = value.clamp(0.0, 1.0);
//         }
//     }

//     pub fn get_weight(&self, index: usize) -> f32 {
//         self.weights.get(index).copied().unwrap_or(0.0)
//     }

//     pub fn reset(&mut self) {
//         self.weights = [0.0; 52];
//     }

//     /// Convert ARKit weights to MH faceshape weights
//     /// Returns vec of (shape_name, weight) for non-zero mapped shapes
//     pub fn to_faceshapes(&self) -> Vec<(&'static str, f32)> {
//         let mapping = arkit_to_faceshapes();
//         let mut result: std::collections::HashMap<&'static str, f32> = std::collections::HashMap::new();

//         for (i, &arkit_weight) in self.weights.iter().enumerate() {
//             if arkit_weight < 0.001 {
//                 continue;
//             }

//             let map = &mapping[i];
//             for &(shape_name, scale) in map.shapes {
//                 let scaled = arkit_weight * scale;
//                 *result.entry(shape_name).or_insert(0.0) += scaled;
//             }
//         }

//         // Clamp and filter
//         result
//             .into_iter()
//             .filter(|(_, w)| *w > 0.001)
//             .map(|(name, w)| (name, w.min(1.0)))
//             .collect()
//     }
// }

// /// All available faceshape names from faceshapes.mxa
// pub const FACESHAPE_NAMES: &[&str] = &[
//     // Visemes (39)
//     "AA", "AE", "AH", "AO", "AW", "AY", "B", "CH", "D", "DH",
//     "EH", "ER", "EY", "F", "G", "H", "IH", "IY", "JH", "K",
//     "L", "M", "N", "NG", "OW", "OY", "P", "R", "S", "SH",
//     "T", "TH", "UH", "UW", "V", "W", "Y", "Z", "ZH",
//     // FACS expressions
//     "blink",
//     "brow_mid_down_left", "brow_mid_down_right",
//     "brow_mid_up_left", "brow_mid_up_right",
//     "brow_outer_down_left", "brow_outer_down_right",
//     "brow_outer_up_left", "brow_outer_up_right",
//     "brow_squeeze",
//     "cheek_balloon_left", "cheek_balloon_right",
//     "cheek_narrow_left", "cheek_narrow_right",
//     "cheek_squint_left", "cheek_squint_right",
//     "cheek_up_left", "cheek_up_right",
//     "lips_lower_in", "lips_lower_out",
//     "lips_mid_lower_down_left", "lips_mid_lower_down_right",
//     "lips_mid_lower_up_left", "lips_mid_lower_up_right",
//     "lips_mid_upper_down_left", "lips_mid_upper_down_right",
//     "lips_mid_upper_up_left", "lips_mid_upper_up_right",
//     "lips_part",
//     "lips_upper_in", "lips_upper_out",
//     "mouth_corner_down_left", "mouth_corner_down_right",
//     "mouth_corner_in_left", "mouth_corner_in_right",
//     "mouth_corner_up_left", "mouth_corner_up_right",
//     "mouth_down_left", "mouth_down_right",
//     "mouth_narrow_left", "mouth_narrow_right",
//     "mouth_open",
//     "mouth_up_left", "mouth_up_right",
//     "mouth_wide_left", "mouth_wide_right",
//     "nose_wrinkle",
//     "rest",
//     "tongue_back_up", "tongue_out", "tongue_up", "tongue_wide",
// ];

// /// Shared faceshapes data resource - loaded once, used by all characters
// #[derive(Resource, Default)]
// pub struct FaceshapeResource {
//     pub handle: Handle<FaceshapesData>,
//     pub loaded: bool,
// }

// /// Component for characters that can have faceshape deformation applied
// /// Stores the base (undeformed) vertex positions and mhid_lookup
// #[derive(Component)]
// pub struct FaceshapeDeform {
//     /// Base vertex positions (before any faceshape deformation)
//     pub base_positions: Vec<Vec3>,
//     /// Maps mesh vertex index → MakeHuman vertex index
//     pub mhid_lookup: Vec<u16>,
// }

// impl FaceshapeDeform {
//     pub fn new(base_positions: Vec<Vec3>, mhid_lookup: Vec<u16>) -> Self {
//         Self { base_positions, mhid_lookup }
//     }
// }

// /// System to apply faceshape deformation based on ExpressionWeights
// /// Modifies mesh vertex positions in-place
// pub fn apply_faceshapes(
//     faceshape_res: Res<FaceshapeResource>,
//     faceshapes_assets: Res<Assets<FaceshapesData>>,
//     query: Query<(&ExpressionWeights, &FaceshapeDeform, &Mesh3d), Changed<ExpressionWeights>>,
//     mut meshes: ResMut<Assets<Mesh>>,
// ) {
//     // Get loaded faceshapes data
//     let Some(faceshapes_data) = faceshapes_assets.get(&faceshape_res.handle) else {
//         return;
//     };

//     for (expr_weights, deform, mesh3d) in query.iter() {
//         // Get mesh to modify
//         let Some(mesh) = meshes.get_mut(&mesh3d.0) else {
//             continue;
//         };

//         // Get shape weights from ARKit expression weights
//         let shape_weights = expr_weights.to_faceshapes();
//         if shape_weights.is_empty() {
//             // Reset to base positions if no shapes active
//             if let Some(VertexAttributeValues::Float32x3(positions)) =
//                 mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
//             {
//                 for (i, pos) in deform.base_positions.iter().enumerate() {
//                     if i < positions.len() {
//                         positions[i] = [pos.x, pos.y, pos.z];
//                     }
//                 }
//             }
//             continue;
//         }

//         // Start with base positions
//         let mut final_positions = deform.base_positions.clone();

//         // Apply each active faceshape
//         for (shape_name, weight) in &shape_weights {
//             if let Some(offsets) = faceshapes_data.get(shape_name) {
//                 // Apply vertex offsets via mhid_lookup
//                 for (mesh_idx, &mh_idx) in deform.mhid_lookup.iter().enumerate() {
//                     if mesh_idx >= final_positions.len() {
//                         break;
//                     }
//                     if let Some(&offset) = offsets.get(&(mh_idx as u32)) {
//                         final_positions[mesh_idx] += offset * *weight;
//                     }
//                 }
//             }
//         }

//         // Write final positions to mesh
//         if let Some(VertexAttributeValues::Float32x3(positions)) =
//             mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
//         {
//             for (i, pos) in final_positions.iter().enumerate() {
//                 if i < positions.len() {
//                     positions[i] = [pos.x, pos.y, pos.z];
//                 }
//             }
//         }
//     }
// }
