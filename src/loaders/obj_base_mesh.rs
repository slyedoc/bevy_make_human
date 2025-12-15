use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    tasks::ConditionalSendFuture,
};
use bevy_obj::{ObjSettings, mesh::load_obj_as_mesh};
use thiserror::Error;

// Unlike normal objPlugin, we need orginal verts as well
#[derive(Asset, TypePath, Debug, Clone)]
pub struct ObjBaseMesh {
    pub mesh: Mesh,
    /// The (makehuman/obj) positions in the base mesh
    pub vertices: Vec<Vec3>,
    /// Mesh vertex idx -> obj vertex idx mapping (for skinning/fitting)
    pub mhid_lookup: Vec<u16>,
}

#[derive(Default)]
pub struct ObjBaseMeshLoader;

#[derive(Debug, Error)]
pub enum ObjBaseMeshLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("OBJ parse error: {0}")]
    Obj(#[from] obj::ObjError),
    #[error("OBJ parse error: {0}")]
    BevyObj(#[from] bevy_obj::mesh::ObjError),
}

impl AssetLoader for ObjBaseMeshLoader {
    type Asset = ObjBaseMesh;
    type Settings = ObjSettings;
    type Error = ObjBaseMeshLoaderError;

    fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let obj_data = obj::ObjData::load_buf(&bytes[..])?;

            let mut mesh = load_obj_as_mesh(&bytes, settings)?;

            // MakeHuman uses DECIMETER units - scale to meters (0.1x)
            // Also flip Z to match Bevy's coordinate system
            const SCALE: f32 = 0.1;

            // Scale mesh verts to match obj verts (bevy_obj doesn't scale)
            // Negate X and Z to convert from MH coords (character faces +Y in Blender export)
            // to Bevy coords (character faces -Z, +X is right)
            if let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            {
                for pos in positions.iter_mut() {
                    pos[0] *= -SCALE; // Negate X to fix left/right
                    pos[1] *= SCALE;
                    pos[2] *= -SCALE;
                }
            }

            // Note: Negating X AND Z = 180° rotation around Y axis (not a reflection)
            // so winding order stays the same (no need to reverse indices)

            // Store obj positions with same transforms (negate X and Z)
            let vertices: Vec<Vec3> = obj_data
                .position
                .iter()
                .map(|&p| Vec3::from(p) * Vec3::new(-SCALE, SCALE, -SCALE))
                .collect();

            // Build mhid_lookup: mesh_vert_idx -> obj_vert_idx
            let mesh_verts: Vec<Vec3> = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|attr| attr.as_float3())
                .map(|positions| positions.iter().map(|p| Vec3::from_array(*p)).collect())
                .unwrap_or_default();

            let mhid_lookup = build_mhid_lookup(&vertices, &mesh_verts);

            Ok(ObjBaseMesh {
                mesh,
                vertices,
                mhid_lookup,
            })
        }
    }
}

/// Build mesh_vert_idx -> obj_vert_idx mapping
fn build_mhid_lookup(obj_verts: &[Vec3], mesh_verts: &[Vec3]) -> Vec<u16> {
    // First build obj_idx -> [mesh_idx...] map
    let mut vertex_map: HashMap<u16, Vec<u16>> = HashMap::default();
    for (mesh_idx, mesh_vert) in mesh_verts.iter().enumerate() {
        for (obj_idx, obj_vert) in obj_verts.iter().enumerate() {
            if (mesh_vert - obj_vert).length() < 0.0001 {
                vertex_map
                    .entry(obj_idx as u16)
                    .or_default()
                    .push(mesh_idx as u16);
                break;
            }
        }
    }

    // Invert to mesh_idx -> obj_idx
    let mut mhid_lookup = vec![0u16; mesh_verts.len()];
    for (obj_idx, mesh_indices) in vertex_map {
        for mesh_idx in mesh_indices {
            mhid_lookup[mesh_idx as usize] = obj_idx;
        }
    }
    mhid_lookup
}

// // Take first object
// let object = &obj_data.objects[0];

// // MakeHuman uses DECIMETER units - scale to meters (0.1x)
// const SCALE: f32 = 0.1;

// // Convert all obj positions to mesh vertices (preserve original indices for mhclo binding)
// let mut positions = Vec::new();
// let mut uvs = Vec::new();
// let mut indices = Vec::new();

// // Preallocate positions array to match obj vertex count
// positions.resize(obj_data.position.len(), [0.0, 0.0, 0.0]);
// uvs.resize(obj_data.position.len(), [0.0, 0.0]);

// // First pass: populate all positions from obj
// for (i, pos) in obj_data.position.iter().enumerate() {
//     positions[i] = [pos[0] * SCALE, pos[1] * SCALE, -pos[2] * SCALE];
// }

// // Second pass: build indices and populate UVs
// for group in &object.groups {
//     for poly in &group.polys {
//         let verts: Vec<_> = poly.0.iter().collect();

//         if verts.len() > 4 {
//             warn!("Skipping polygon with {} vertices", verts.len());
//         }

//         // Process triangle or quad
//         let tri_indices = if verts.len() == 3 {
//             vec![(0, 1, 2)]
//         } else if verts.len() == 4 {
//             // Quad → two triangles
//             vec![(0, 1, 2), (0, 2, 3)]
//         } else {
//             continue; // Skip non-tri/quad
//         };

//         for (i0, i1, i2) in tri_indices {
//             // Reverse winding because Z-flip changes handedness
//             for &vert_idx in &[i0, i2, i1] {
//                 let vert = &verts[vert_idx];
//                 let pos_idx = vert.0 as u32;
//                 let uv_idx = vert.1;

//                 // Store UV if available
//                 if let Some(uv_idx) = uv_idx {
//                     uvs[pos_idx as usize] = [
//                         obj_data.texture[uv_idx][0],
//                         1.0 - obj_data.texture[uv_idx][1],
//                     ];
//                 }

//                 indices.push(pos_idx);
//             }
//         }
//     }
// }

// // Ensure UVs have same length as positions
// while uvs.len() < positions.len() {
//     uvs.push([0.0, 0.0]);
// }

// let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
// mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
// if !uvs.is_empty() {
//     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
// }
// mesh.insert_indices(Indices::U32(indices));

// // TODO: Come back to work on normals
// // Now that vertices are deduplicated, use Bevy's area-weighted normals
// mesh.compute_area_weighted_normals();
