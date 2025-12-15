//! Skeleton and bone structures for MakeHuman characters

use bevy::{math::Affine3A, platform::collections::HashMap, prelude::*};

/// Component storing character skeleton - bones, hierarchy, bind pose
#[derive(Component, Clone)]
pub struct Skeleton {
    /// Bone definitions (name, head, tail, roll)
    pub bones: Vec<Bone>,
    /// Parent indices - hierarchy[i] = parent bone index for bone i
    pub hierarchy: Vec<Option<usize>>,
    /// Bind pose transforms (T-pose) - LOCAL space
    pub bind_pose: Vec<Transform>,
    /// Global bind rotations - for converting world-space to local-space animations
    pub global_bind_rotations: Vec<Quat>,
    /// Inverse bind pose matrices (for skinning)
    pub inverse_bind_matrices: Vec<Mat4>,
    /// Bone name → index lookup
    pub bone_indices: HashMap<String, usize>,
}

/// Single bone definition
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub head: Vec3, // Start position in mesh space
    pub tail: Vec3, // End position in mesh space
    pub roll: f32,  // Twist rotation around bone axis (radians)
}

impl Bone {
    /// Get bone direction vector (normalized)
    pub fn direction(&self) -> Vec3 {
        (self.tail - self.head).normalize()
    }

    /// Get bone length
    pub fn length(&self) -> f32 {
        self.head.distance(self.tail)
    }

    /// Create transform for this bone in bind pose using base rotation from skeleton GLB
    ///
    /// This uses Humentity's formula:
    /// 1. base_rot defines the bone's local coordinate system orientation
    /// 2. We correct it to point along the actual bone direction (head->tail)
    /// 3. Final rotation = correction * base_rot
    ///
    /// For zero-length bones (like Armature root), we use base_rot directly
    /// since there's no meaningful bone direction to correct to.
    ///
    /// The base_rot comes from a reference skeleton GLB exported from Blender
    /// which has correct orientations for animation compatibility.
    pub fn bind_transform_with_base(&self, base_rot: Quat) -> Transform {
        let length = self.length();

        // For zero/tiny length bones (coordinate transform bones like Armature),
        // use the base rotation directly without correction
        let rotation = if length < 1e-4 {
            base_rot
        } else {
            let orientation = self.direction();
            // Correct from base rotation's Y-axis to actual bone direction
            let correction = Quat::from_rotation_arc(base_rot * Vec3::Y, orientation);
            correction * base_rot
        };

        Transform {
            translation: self.head,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Create transform for this bone in bind pose (fallback without base rotation)
    ///
    /// Blender/MH bone convention:
    /// - Bones point along local Y axis (head to tail)
    /// - Roll rotates around the bone's Y axis (the bone direction)
    pub fn bind_transform(&self) -> Transform {
        let translation = self.head;
        let direction = self.direction();

        // Create rotation from Y-axis (bone's default direction) to actual direction
        let rotation = if direction.abs_diff_eq(Vec3::Y, 1e-6) {
            Quat::IDENTITY
        } else if direction.abs_diff_eq(-Vec3::Y, 1e-6) {
            Quat::from_rotation_z(std::f32::consts::PI)
        } else {
            Quat::from_rotation_arc(Vec3::Y, direction)
        };

        // Apply roll (twist around bone axis in local space)
        let roll_quat = Quat::from_axis_angle(Vec3::Y, self.roll);
        let final_rotation = rotation * roll_quat;

        Transform {
            translation,
            rotation: final_rotation,
            scale: Vec3::ONE,
        }
    }
}

impl Skeleton {
    /// Create new skeleton from bones and hierarchy
    pub fn new(bones: Vec<Bone>, hierarchy: Vec<Option<usize>>) -> Self {
        Self::new_internal(bones, hierarchy, None)
    }

    /// Create skeleton using base rotations from a reference skeleton GLB
    pub fn new_with_base_rotations(
        bones: Vec<Bone>,
        hierarchy: Vec<Option<usize>>,
        base_rotations: &HashMap<String, Quat>,
    ) -> Self {
        Self::new_internal(bones, hierarchy, Some(base_rotations))
    }

    fn new_internal(
        bones: Vec<Bone>,
        hierarchy: Vec<Option<usize>>,
        base_rotations: Option<&HashMap<String, Quat>>,
    ) -> Self {
        assert_eq!(
            bones.len(),
            hierarchy.len(),
            "Bones and hierarchy must have same length"
        );

        // Build bone name → index lookup
        let bone_indices: HashMap<String, usize> = bones
            .iter()
            .enumerate()
            .map(|(i, bone)| (bone.name.clone(), i))
            .collect();

        // Calculate GLOBAL bind pose transforms
        // If base rotations provided, use Humentity's formula for animation compatibility
        let global_bind_pose: Vec<Transform> = bones
            .iter()
            .map(|bone| {
                if let Some(base_rots) = base_rotations {
                    if let Some(&base_rot) = base_rots.get(&bone.name) {
                        // Use Humentity's formula: position from MH, rotation corrected from base
                        bone.bind_transform_with_base(base_rot)
                    } else {
                        // Fallback to computed rotation if bone not in reference
                        bone.bind_transform()
                    }
                } else {
                    bone.bind_transform()
                }
            })
            .collect();

        // Store global bind rotations for animation conversion
        let global_bind_rotations: Vec<Quat> =
            global_bind_pose.iter().map(|t| t.rotation).collect();

        // Calculate LOCAL transforms relative to parent
        // For skinning entities in a hierarchy, we need local transforms
        let mut bind_pose = vec![Transform::IDENTITY; bones.len()];
        for (bone_idx, &parent_idx_opt) in hierarchy.iter().enumerate() {
            let global = global_bind_pose[bone_idx];
            if let Some(parent_idx) = parent_idx_opt {
                // Local = inverse(parent_global) * global
                let parent_global = global_bind_pose[parent_idx];
                let parent_inv = parent_global.compute_affine().inverse();
                let local_affine = parent_inv * global.compute_affine();
                bind_pose[bone_idx] = Transform::from_matrix(Mat4::from(local_affine));
            } else {
                // Root bone - local == global
                bind_pose[bone_idx] = global;
            }
        }

        // Calculate inverse bind matrices from GLOBAL transforms
        // This is what GPU skinning needs: inverse of world-space bind pose
        let inverse_bind_matrices: Vec<Mat4> = global_bind_pose
            .iter()
            .map(|transform| {
                let affine = Affine3A::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );
                Mat4::from(affine.inverse())
            })
            .collect();

        Self {
            bones,
            hierarchy,
            bind_pose,
            global_bind_rotations,
            inverse_bind_matrices,
            bone_indices,
        }
    }

    /// Find bone index by name
    pub fn bone_index(&self, name: &str) -> Option<usize> {
        self.bone_indices.get(name).copied()
    }

    /// Get bone by name
    pub fn bone(&self, name: &str) -> Option<&Bone> {
        self.bone_index(name).map(|idx| &self.bones[idx])
    }

    /// Apply reference rotations from GLTF to update bind poses
    ///
    /// This recalculates local transforms and inverse bind matrices using
    /// the reference global rotations while keeping the original bone positions.
    /// Use this to make animations compatible when the skeleton was built
    /// without reference rotations.
    pub fn apply_reference_rotations(&mut self, ref_rotations: &HashMap<String, Quat>) {
        // Build new global bind pose using MH positions + GLTF rotations
        let global_bind_pose: Vec<Transform> = self
            .bones
            .iter()
            .map(|bone| {
                if let Some(&ref_rot) = ref_rotations.get(&bone.name) {
                    Transform {
                        translation: bone.head,
                        rotation: ref_rot,
                        scale: Vec3::ONE,
                    }
                } else {
                    bone.bind_transform()
                }
            })
            .collect();

        // Update global bind rotations
        self.global_bind_rotations = global_bind_pose.iter().map(|t| t.rotation).collect();

        // Recalculate local transforms
        for (bone_idx, &parent_idx_opt) in self.hierarchy.iter().enumerate() {
            let global = global_bind_pose[bone_idx];
            if let Some(parent_idx) = parent_idx_opt {
                let parent_global = global_bind_pose[parent_idx];
                let parent_inv = parent_global.compute_affine().inverse();
                let local_affine = parent_inv * global.compute_affine();
                self.bind_pose[bone_idx] = Transform::from_matrix(Mat4::from(local_affine));
            } else {
                self.bind_pose[bone_idx] = global;
            }
        }

        // Recalculate inverse bind matrices
        self.inverse_bind_matrices = global_bind_pose
            .iter()
            .map(|transform| {
                let affine = Affine3A::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.translation,
                );
                Mat4::from(affine.inverse())
            })
            .collect();
    }

    /// Get global transform for bone (parent chain multiplied)
    pub fn global_transform(&self, bone_idx: usize, local_transforms: &[Transform]) -> Transform {
        let mut transform = local_transforms[bone_idx];
        let mut current = bone_idx;

        // Walk up parent chain, multiplying transforms
        while let Some(parent_idx) = self.hierarchy[current] {
            transform = local_transforms[parent_idx] * transform;
            current = parent_idx;
        }

        transform
    }

    /// Apply pose (local bone transforms) and compute global transforms
    pub fn compute_global_transforms(&self, local_transforms: &[Transform]) -> Vec<Mat4> {
        let mut global_matrices = vec![Mat4::IDENTITY; self.bones.len()];

        for bone_idx in 0..self.bones.len() {
            let global_transform = self.global_transform(bone_idx, local_transforms);
            let affine = Affine3A::from_scale_rotation_translation(
                global_transform.scale,
                global_transform.rotation,
                global_transform.translation,
            );
            global_matrices[bone_idx] = Mat4::from(affine);
        }

        global_matrices
    }

    /// Apply pose and compute skinning matrices (global * inverse_bind)
    pub fn compute_skinning_matrices(&self, local_transforms: &[Transform]) -> Vec<Mat4> {
        let global_matrices = self.compute_global_transforms(local_transforms);

        global_matrices
            .iter()
            .zip(&self.inverse_bind_matrices)
            .map(|(global, inv_bind)| *global * *inv_bind)
            .collect()
    }

    /// Apply skinning to mesh vertices (CPU implementation)
    ///
    /// vertex_weights: per-vertex list of (bone_idx, weight) pairs
    /// bind_vertices: original vertex positions in bind pose
    /// local_transforms: current bone transforms (pose)
    ///
    /// Returns: deformed vertex positions
    pub fn apply_skinning(
        &self,
        vertex_weights: &[Vec<(usize, f32)>],
        bind_vertices: &[Vec3],
        local_transforms: &[Transform],
    ) -> Vec<Vec3> {
        let skinning_matrices = self.compute_skinning_matrices(local_transforms);

        bind_vertices
            .iter()
            .zip(vertex_weights)
            .map(|(&bind_pos, weights)| {
                if weights.is_empty() {
                    // No skinning weights, use bind position
                    return bind_pos;
                }

                // Apply weighted blend of bone transforms
                let mut skinned_pos = Vec3::ZERO;
                for &(bone_idx, weight) in weights {
                    if bone_idx < skinning_matrices.len() && weight > 1e-6 {
                        // Transform vertex by bone matrix
                        let transformed = skinning_matrices[bone_idx].transform_point3(bind_pos);
                        skinned_pos += transformed * weight;
                    }
                }

                skinned_pos
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_direction() {
        let bone = Bone {
            name: "test".to_string(),
            head: Vec3::ZERO,
            tail: Vec3::new(0.0, 1.0, 0.0),
            roll: 0.0,
        };

        assert_eq!(bone.direction(), Vec3::Y);
        assert_eq!(bone.length(), 1.0);
    }

    #[test]
    fn test_skeleton_hierarchy() {
        let bones = vec![
            Bone {
                name: "root".to_string(),
                head: Vec3::ZERO,
                tail: Vec3::Y,
                roll: 0.0,
            },
            Bone {
                name: "child".to_string(),
                head: Vec3::Y,
                tail: Vec3::new(0.0, 2.0, 0.0),
                roll: 0.0,
            },
        ];

        let hierarchy = vec![None, Some(0)]; // child's parent is root
        let skeleton = Skeleton::new(bones, hierarchy);

        assert_eq!(skeleton.bones.len(), 2);
        assert_eq!(skeleton.bone_index("root"), Some(0));
        assert_eq!(skeleton.bone_index("child"), Some(1));
        assert_eq!(skeleton.hierarchy[1], Some(0));
    }
}
