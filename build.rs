use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

// BEWLOW are all the same 
const COMMON_ITEMS: [&str; 4] = ["mhclo", "mhmat", "obj",  "thumb"];

fn main() -> io::Result<()> {
    
    // for (key, value) in env::vars() {
    //     println!("cargo:warning=Env {}={}", key, value);
    // }

    let assets_dir = get_base_path().join("assets").join("make_human");
    println!("cargo:warning=Env {:?}", assets_dir.as_os_str());
    println!("cargo:rerun-if-changed=build.rs");    
    println!("cargo:rerun-if-changed={:?}", assets_dir.as_os_str());
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("assets.rs");
    let mut f = File::create(dest_path)?;

    // Import Component for enums that derive it
    writeln!(f, "use bevy::prelude::Component;")?;
    writeln!(f)?;

    // Proxymeshes -> SkinMesh
    generate_asset_enum(&mut f, &assets_dir, "proxymeshes", "SkinMesh", &AssetFilePattern {
        required: &["obj", "proxy", "thumb"],
        textures: &[],
    })?;

    // Rigs
    generate_rig_enum(&mut f, &assets_dir)?;

    // Skins -> SkinMaterial
    generate_asset_enum(&mut f, &assets_dir, "skins", "SkinMaterial", &AssetFilePattern {
        required: &["mhmat", "thumb"],
        textures: &["diffuse", "normal", "specular"],
    })?;

    // Generate enums for each asset type with specific file patterns
    generate_asset_enum(&mut f, &assets_dir, "hair", "Hair", &AssetFilePattern {
        required: &["mhclo", "mhmat", "obj", "thumb"],
        textures: &["diffuse", "normal", "specular", "ao", "bump"],
    })?;

    // Clothing: .mhclo, .mhmat required; .obj extracted from mhclo parsing; textures optional
    generate_asset_enum(&mut f, &assets_dir, "clothes", "ClothingAsset", &AssetFilePattern {
        required: &["mhclo", "mhmat", "obj", "thumb"],
        textures: &["diffuse", "normal", "specular", "ao", "bump"],
    })?;

    // Eyes
    generate_flat_file_enum(&mut f, &assets_dir, "eyes/materials", "EyesMaterial", "mhmat")?;
    generate_asset_enum(&mut f, &assets_dir, "eyes", "EyesMesh", &AssetFilePattern {
        required: &["mhclo", "obj", "thumb"],
        textures: &[],
    })?;    
  
    // Eyebrows/Eyelashes: .mhclo, .obj, .mhmat, .thumb
    generate_asset_enum(&mut f, &assets_dir, "eyebrows", "EyebrowsAsset", &AssetFilePattern {
        required: &COMMON_ITEMS,
        textures: &[],
    })?;

    // Eyelashes
    generate_asset_enum(&mut f, &assets_dir, "eyelashes", "EyelashesAsset", &AssetFilePattern {
        required: &COMMON_ITEMS,
        textures: &[],
    })?;

    // Teeth
    generate_asset_enum(&mut f, &assets_dir, "teeth", "TeethAsset", &AssetFilePattern {
        required: &COMMON_ITEMS,
        textures: &[],
    })?;

    // Tongue
    generate_asset_enum(&mut f, &assets_dir, "tongue", "TongueAsset", &AssetFilePattern {
        required: &COMMON_ITEMS,
        textures: &[],
    })?;

    // Poses - BVH files for static poses
    generate_pose_enum(&mut f, &assets_dir)?;

    // Morph targets - read from target.json
    generate_morph_enums(&mut f, &assets_dir)?;

    // Expression targets - for facial animation (ARKit compatible)
    generate_expression_enum(&mut f, &assets_dir)?;

    // Phenotype - macrodetails for race/gender/age/muscle/weight etc
    generate_phenotype(&mut f, &assets_dir)?;

    Ok(())
}

// from bevy_asset
pub(crate) fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("BEVY_ASSET_ROOT") {
        PathBuf::from(manifest_dir)
    } else if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}

struct AssetFilePattern {
    required: &'static [&'static str],    
    textures: &'static [&'static str],
}

fn generate_asset_enum(
    f: &mut File,
    assets_dir: &Path,
    subdir: &str,
    enum_name: &str,
    pattern: &AssetFilePattern,
) -> io::Result<()> {
    let dir_path = assets_dir.join(subdir);

    if !dir_path.exists() {
        println!("cargo:warning=Skipping {}, directory not found", subdir);
        return Ok(());
    }

    // Collect all subdirectories (each contains one asset)
    let mut entries: Vec<_> = fs::read_dir(&dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by(|a, b| {
        a.path()
            .file_name()
            .cmp(&b.path().file_name())
    });

    // Write enum with strum derives including EnumProperty
    // Add Component derive for types used directly as components (Hair)
    let component_derive = if enum_name == "Hair" { "Component, " } else { "" };
    writeln!(f, "/// Generated from assets/make_human/{}", subdir)?;
    writeln!(f, "#[derive({}Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect)]", component_derive)?;
    writeln!(f, "pub enum {} {{", enum_name)?;

    for entry in &entries {
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        let asset_dir = entry.path();

        // Check if this directory has at least one required file
        let has_required = pattern.required.iter().any(|file_type| {
            asset_dir.join(format!("{}.{}", dir_name_str, file_type)).exists()
        });

        if !has_required {
            continue;
        }

        let variant_name = sanitize_name(&dir_name_str);

        // Scan for all files in this asset directory
        let mut props = Vec::new();
        // Required files - find and assert they exist
        for file_type in pattern.required {
            // Try exact match first
            let mut file_path = asset_dir.join(format!("{}.{}", dir_name_str, file_type));

            // Fallback: search for ANY file with this extension in the directory
            if !file_path.exists() {
                let found = fs::read_dir(&asset_dir).ok()
                    .and_then(|entries| {
                        entries.filter_map(|e| e.ok())
                            .find(|e| {
                                e.path().extension()
                                    .and_then(|ext| ext.to_str())
                                    .map(|ext| ext == *file_type)
                                    .unwrap_or(false)
                            })
                    });

                if let Some(found_entry) = found {
                    file_path = found_entry.path();
                } else {
                    panic!(
                        "Required file missing for {}: {} (expected at {:?}, or any *.{} in dir)",
                        enum_name, file_type, asset_dir, file_type
                    );
                }
            }

            let filename = file_path.file_name().unwrap().to_string_lossy();
            let path = format!("make_human/{}/{}/{}", subdir, dir_name_str, filename);
            let clean_name = file_type.replace(".", "_");
            props.push(format!("{} = \"{}\"", clean_name, path));
        }

        // Textures - scan for common texture files
        for texture_type in pattern.textures {
            let file_path = asset_dir.join(format!("{}_{}.png", dir_name_str, texture_type));
            if file_path.exists() {
                let png_path = format!("make_human/{}/{}/{}_{}.png", subdir, dir_name_str, dir_name_str, texture_type);
                props.push(format!("{} = \"{}\"", texture_type, png_path));
            }
        }

        // For proxies, add vertex count to doc
        let doc_suffix = if enum_name == "ProxyMesh" {
            // Count verts from .obj file
            let obj_path = asset_dir.join(format!("{}.obj", dir_name_str));
            if obj_path.exists() {
                if let Ok(content) = fs::read_to_string(&obj_path) {
                    let vert_count = content.lines()
                        .filter(|line| line.starts_with("v "))
                        .count();
                    format!(" ({} verts)", vert_count)
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        writeln!(f, "    /// {}{}", dir_name_str, doc_suffix)?;
        writeln!(f, "    #[strum(props({}))]", props.join(", "))?;
        writeln!(f, "    {},", variant_name)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    // Generate helper methods using EnumProperty
    writeln!(f, "impl {} {{", enum_name)?;

    // Required file accessors
    for file_type in pattern.required {
        let clean_name = file_type.replace(".", "_",);

        writeln!(f, "    /// Get .{} path", file_type)?;
        writeln!(f, "    pub fn {}_path(&self) -> &str {{", clean_name)?;
        writeln!(f, "        self.get_str(\"{}\").unwrap()", clean_name)?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
    }

    // Texture accessors
    for texture_type in pattern.textures {
        writeln!(f, "    /// Get {} texture path if available", texture_type)?;
        writeln!(f, "    pub fn {}_texture(&self) -> Option<&str> {{", texture_type)?;
        writeln!(f, "        use strum::EnumProperty;")?;
        writeln!(f, "        self.get_str(\"{}\")", texture_type)?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

fn generate_flat_file_enum(
    f: &mut File,
    assets_dir: &Path,
    subdir: &str,
    enum_name: &str,
    extension: &str,
) -> io::Result<()> {
    let dir_path = assets_dir.join(subdir);

    if !dir_path.exists() {
        println!("cargo:warning=Skipping {}, directory not found", subdir);
        return Ok(());
    }

    // Collect all files with extension
    let mut entries: Vec<_> = fs::read_dir(&dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() &&
            e.path().extension().and_then(|s| s.to_str()) == Some(extension)
        })
        .collect();

    entries.sort_by(|a, b| {
        a.path().file_stem().cmp(&b.path().file_stem())
    });

    // Write enum
    writeln!(f, "/// Generated from assets/make_human/{}", subdir)?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect)]")?;
    writeln!(f, "pub enum {} {{", enum_name)?;

    for entry in &entries {
        let path = entry.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let file_name = entry.file_name().to_string_lossy().to_string();
        let variant_name = sanitize_name(&file_stem);
        let full_path = format!("make_human/{}/{}", subdir, file_name);

        writeln!(f, "    /// {}", file_stem)?;
        writeln!(f, "    #[strum(props(mhmat = \"{}\"))]", full_path)?;
        writeln!(f, "    {},", variant_name)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    // Generate helper methods
    writeln!(f, "impl {} {{", enum_name)?;
    writeln!(f, "    /// Get .mhmat path")?;
    writeln!(f, "    pub fn mhmat_path(&self) -> &str {{")?;
    writeln!(f, "        self.get_str(\"mhmat\").unwrap()")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

/// Generate PoseAsset enum from poses directory (BVH files)
fn generate_pose_enum(f: &mut File, assets_dir: &Path) -> io::Result<()> {
    let dir_path = assets_dir.join("poses");

    if !dir_path.exists() {
        println!("cargo:warning=Skipping poses, directory not found");
        return Ok(());
    }

    // Collect all subdirectories containing .bvh files
    let mut entries: Vec<_> = fs::read_dir(&dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            if !path.is_dir() {
                return false;
            }
            // Check for .bvh file in this directory
            let dir_name = e.file_name();
            let bvh_path = path.join(format!("{}.bvh", dir_name.to_string_lossy()));
            bvh_path.exists()
        })
        .collect();

    entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

    writeln!(f, "/// Generated from assets/make_human/poses")?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect)]")?;
    writeln!(f, "pub enum PoseAsset {{")?;

    for entry in &entries {
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        let variant_name = sanitize_name(&dir_name_str);
        let bvh_path = format!("make_human/poses/{}/{}.bvh", dir_name_str, dir_name_str);
        let thumb_path = format!("make_human/poses/{}/{}.thumb", dir_name_str, dir_name_str);

        writeln!(f, "    /// {}", dir_name_str)?;
        writeln!(f, "    #[strum(props(bvh = \"{}\", thumb = \"{}\"))]", bvh_path, thumb_path)?;
        writeln!(f, "    {},", variant_name)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    // Generate helper methods
    writeln!(f, "impl PoseAsset {{")?;
    writeln!(f, "    /// Get .bvh path")?;
    writeln!(f, "    pub fn bvh_path(&self) -> &'static str {{")?;
    writeln!(f, "        self.get_str(\"bvh\").unwrap()")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

/// Generate Rig enum - requires rig.json and weights.json, optional skeleton.glb
fn generate_rig_enum(f: &mut File, assets_dir: &Path) -> io::Result<()> {
    let dir_path = assets_dir.join("rigs/standard");

    if !dir_path.exists() {
        println!("cargo:warning=Skipping rigs/standard, directory not found");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(&dir_path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();

    entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

    writeln!(f, "/// Generated from assets/make_human/rigs/standard")?;
    writeln!(f, "#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect)]")?;
    writeln!(f, "pub enum Rig {{")?;

    let mut first = true;
    for entry in &entries {
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        let asset_dir = entry.path();

        // Check required files exist
        let rig_path = asset_dir.join(format!("{}.rig.json", dir_name_str));
        let weights_path = asset_dir.join(format!("{}.weights.json", dir_name_str));

        if !rig_path.exists() || !weights_path.exists() {
            continue;
        }

        let variant_name = sanitize_name(&dir_name_str);

        // Mark first variant as default
        if first {
            writeln!(f, "    #[default]")?;
            first = false;
        }
        let mut props = Vec::new();

        props.push(format!("rig_json = \"make_human/rigs/standard/{}/{}.rig.json\"", dir_name_str, dir_name_str));
        props.push(format!("weights_json = \"make_human/rigs/standard/{}/{}.weights.json\"", dir_name_str, dir_name_str));

        // Check for optional skeleton GLB (contains base rotations for animation)
        let glb_path = asset_dir.join(format!("{}.glb", dir_name_str));
        if glb_path.exists() {
            props.push(format!("skeleton_glb = \"make_human/rigs/standard/{}/{}.glb\"", dir_name_str, dir_name_str));
        }

        writeln!(f, "    /// {}", dir_name_str)?;
        writeln!(f, "    #[strum(props({}))]", props.join(", "))?;
        writeln!(f, "    {},", variant_name)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    // Generate helper methods
    writeln!(f, "impl Rig {{")?;

    writeln!(f, "    /// Get .rig.json path")?;
    writeln!(f, "    pub fn rig_json_path(&self) -> &str {{")?;
    writeln!(f, "        self.get_str(\"rig_json\").unwrap()")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Get .weights.json path")?;
    writeln!(f, "    pub fn weights_json_path(&self) -> &str {{")?;
    writeln!(f, "        self.get_str(\"weights_json\").unwrap()")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Get skeleton .glb path if available (for base rotations)")?;
    writeln!(f, "    pub fn skeleton_glb_path(&self) -> Option<&str> {{")?;
    writeln!(f, "        self.get_str(\"skeleton_glb\")")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

fn sanitize_name(name: &str) -> String {
    name.split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .unwrap()
                .to_uppercase()
                .chain(chars.flat_map(|c| c.to_lowercase()))
                .collect::<String>()
        })
        .collect()
}

/// Pair type determines which suffix means negative vs positive
#[derive(Debug, Clone, Copy, PartialEq)]
enum PairType {
    IncrDecr,       // neg=decr, pos=incr
    UpDown,         // neg=down, pos=up
    InOut,          // neg=in, pos=out
    ForwardBackward,// neg=backward, pos=forward
    ConvexConcave,  // neg=concave, pos=convex
    CompressUncompress, // neg=compress, pos=uncompress
    SquareRound,    // neg=square, pos=round
    PointedTriangle,// neg=pointed, pos=triangle
    Single,         // only one target (0..1 range)
}

impl PairType {
    fn from_label(label: &str) -> Self {
        if label.ends_with("-decr-incr") { Self::IncrDecr }
        else if label.ends_with("-down-up") { Self::UpDown }
        else if label.ends_with("-in-out") { Self::InOut }
        else if label.ends_with("-backward-forward") { Self::ForwardBackward }
        else if label.ends_with("-concave-convex") || label.ends_with("-convex-concave") { Self::ConvexConcave }
        else if label.ends_with("-compress-uncompress") { Self::CompressUncompress }
        else if label.ends_with("-square-round") { Self::SquareRound }
        else if label.ends_with("-pointed-triangle") { Self::PointedTriangle }
        else { Self::Single }
    }

}

/// Represents a morph (binary pair or single)
#[derive(Debug, Clone)]
struct MorphEntry {
    /// Base name (e.g., "upperarm-fat-l" for sided, "head-age" for unsided)
    name: String,
    /// Pair type determines value range and path selection
    pair_type: PairType,
    /// Path to negative target (for binary pairs)
    neg_path: Option<String>,
    /// Path to positive target (for binary pairs) or only path (for singles)
    pos_path: Option<String>,
}

fn generate_morph_enums(f: &mut File, assets_dir: &Path) -> io::Result<()> {
    // Read target.json to get categories
    let target_json_path = assets_dir.join("targets/target.json");
    let json_str = fs::read_to_string(&target_json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Failed to parse target.json");

    let categories = json.as_object()
        .expect("target.json must be object");

    let mut category_names: Vec<String> = categories.keys()
        .filter(|k| *k != "measure") // skip measure category
        .cloned()
        .collect();
    category_names.sort();

    // Collect morph entries per category
    let mut entries_by_category: HashMap<String, Vec<MorphEntry>> = HashMap::new();

    for (category, data) in categories {
        if category == "measure" {
            continue;
        }

        let mut entries: Vec<MorphEntry> = Vec::new();
        let mut seen_bases: std::collections::HashSet<String> = std::collections::HashSet::new();

        if let Some(cats) = data.get("categories").and_then(|v| v.as_array()) {
            for cat in cats {
                let label = cat.get("label").and_then(|v| v.as_str()).unwrap_or("");
                let pair_type = PairType::from_label(label);
                let opposites = cat.get("opposites");

                if pair_type == PairType::Single {
                    // Single target (no opposites or empty opposites)
                    if let Some(targets) = cat.get("targets").and_then(|v| v.as_array()) {
                        for target in targets {
                            if let Some(target_name) = target.as_str() {
                                if !seen_bases.contains(target_name) {
                                    seen_bases.insert(target_name.to_string());
                                    entries.push(MorphEntry {
                                        name: target_name.to_string(),
                                        pair_type: PairType::Single,
                                        neg_path: None,
                                        pos_path: Some(format!("make_human/targets/{}/{}.target", category, target_name)),
                                    });
                                }
                            }
                        }
                    }
                } else if let Some(opp) = opposites.and_then(|v| v.as_object()) {
                    // Binary pair with opposites
                    let pos_left = opp.get("positive-left").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                    let neg_left = opp.get("negative-left").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                    let pos_right = opp.get("positive-right").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                    let neg_right = opp.get("negative-right").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                    let pos_unsided = opp.get("positive-unsided").and_then(|v| v.as_str()).filter(|s| !s.is_empty());
                    let neg_unsided = opp.get("negative-unsided").and_then(|v| v.as_str()).filter(|s| !s.is_empty());

                    // Left side - use suffix L
                    if pos_left.is_some() || neg_left.is_some() {
                        let base = extract_base_name(pos_left.or(neg_left).unwrap(), pair_type);
                        let sided_name = format!("{}-l", base);
                        if !seen_bases.contains(&sided_name) {
                            seen_bases.insert(sided_name.clone());
                            entries.push(MorphEntry {
                                name: sided_name,
                                pair_type,
                                neg_path: neg_left.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                                pos_path: pos_left.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                            });
                        }
                    }

                    // Right side - use suffix R
                    if pos_right.is_some() || neg_right.is_some() {
                        let base = extract_base_name(pos_right.or(neg_right).unwrap(), pair_type);
                        let sided_name = format!("{}-r", base);
                        if !seen_bases.contains(&sided_name) {
                            seen_bases.insert(sided_name.clone());
                            entries.push(MorphEntry {
                                name: sided_name,
                                pair_type,
                                neg_path: neg_right.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                                pos_path: pos_right.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                            });
                        }
                    }

                    // Unsided
                    if pos_unsided.is_some() || neg_unsided.is_some() {
                        let base = extract_base_name(pos_unsided.or(neg_unsided).unwrap(), pair_type);
                        if !seen_bases.contains(&base) {
                            seen_bases.insert(base.clone());
                            entries.push(MorphEntry {
                                name: base,
                                pair_type,
                                neg_path: neg_unsided.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                                pos_path: pos_unsided.map(|t| format!("make_human/targets/{}/{}.target", category, t)),
                            });
                        }
                    }
                }
            }
        }

        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries_by_category.insert(category.clone(), entries);
    }

    // Generate enum per category
    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();

        if entries.is_empty() {
            continue;
        }

        let enum_name = format!("{}Morph", sanitize_name(category));

        writeln!(f, "/// Generated from target.json category: {}", category)?;
        writeln!(f, "/// Binary pairs: -1.0 to 1.0 (neg/pos targets)")?;
        writeln!(f, "/// Singles: 0.0 to 1.0 (one target)")?;
        writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, EnumProperty, Reflect)]")?;
        writeln!(f, "pub enum {} {{", enum_name)?;

        for entry in entries {
            let variant_name = sanitize_name(&entry.name);
            let mut props = Vec::new();

            if let Some(ref path) = entry.pos_path {
                props.push(format!("pos = \"{}\"", path));
            }
            if let Some(ref path) = entry.neg_path {
                props.push(format!("neg = \"{}\"", path));
            }
            if entry.pair_type == PairType::Single {
                props.push("single = \"true\"".to_string());
            }

            let range = if entry.pair_type == PairType::Single { "0.0 to 1.0" } else { "-1.0 to 1.0" };
            writeln!(f, "    /// {} ({})", entry.name, range)?;
            if !props.is_empty() {
                writeln!(f, "    #[strum(props({}))]", props.join(", "))?;
            }
            writeln!(f, "    {},", variant_name)?;
        }

        writeln!(f, "}}")?;
        writeln!(f)?;

        // Impl
        writeln!(f, "impl {} {{", enum_name)?;
        writeln!(f, "    /// Get positive target path")?;
        writeln!(f, "    pub fn pos_path(&self) -> Option<&str> {{")?;
        writeln!(f, "        self.get_str(\"pos\")")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    /// Get negative target path")?;
        writeln!(f, "    pub fn neg_path(&self) -> Option<&str> {{")?;
        writeln!(f, "        self.get_str(\"neg\")")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    /// Check if this is a single (0..1) vs binary (-1..1)")?;
        writeln!(f, "    pub fn is_single(&self) -> bool {{")?;
        writeln!(f, "        self.get_str(\"single\").is_some()")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    /// Get target path based on sign of value")?;
        writeln!(f, "    /// For singles, always returns pos_path")?;
        writeln!(f, "    /// For binary, neg value -> neg_path, pos value -> pos_path")?;
        writeln!(f, "    pub fn target_path(&self, value: f32) -> Option<&str> {{")?;
        writeln!(f, "        if self.is_single() || value >= 0.0 {{ self.pos_path() }} else {{ self.neg_path() }}")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    /// Get valid value range: (min, max)")?;
        writeln!(f, "    pub fn value_range(&self) -> (f32, f32) {{")?;
        writeln!(f, "        if self.is_single() {{ (0.0, 1.0) }} else {{ (-1.0, 1.0) }}")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "}}")?;
        writeln!(f)?;
    }

    // Generate unified MorphTarget enum
    writeln!(f, "/// Unified morph target enum containing all categories")?;
    writeln!(f, "/// Binary pairs: -1.0 to 1.0, Singles: 0.0 to 1.0")?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]")?;
    writeln!(f, "pub enum MorphTarget {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let enum_name = format!("{}Morph", sanitize_name(category));
        let variant = sanitize_name(category);
        writeln!(f, "    {}({}),", variant, enum_name)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    writeln!(f, "impl MorphTarget {{")?;
    writeln!(f, "    /// Get target path based on sign of value")?;
    writeln!(f, "    pub fn target_path(&self, value: f32) -> Option<&str> {{")?;
    writeln!(f, "        match self {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        writeln!(f, "            Self::{}(m) => m.target_path(value),", variant)?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Get positive target path")?;
    writeln!(f, "    pub fn pos_path(&self) -> Option<&str> {{")?;
    writeln!(f, "        match self {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        writeln!(f, "            Self::{}(m) => m.pos_path(),", variant)?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Get negative target path")?;
    writeln!(f, "    pub fn neg_path(&self) -> Option<&str> {{")?;
    writeln!(f, "        match self {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        writeln!(f, "            Self::{}(m) => m.neg_path(),", variant)?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Check if this is a single (0..1) vs binary (-1..1)")?;
    writeln!(f, "    pub fn is_single(&self) -> bool {{")?;
    writeln!(f, "        match self {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        writeln!(f, "            Self::{}(m) => m.is_single(),", variant)?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    writeln!(f, "    /// Get valid value range: (min, max)")?;
    writeln!(f, "    pub fn value_range(&self) -> (f32, f32) {{")?;
    writeln!(f, "        match self {{")?;

    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        writeln!(f, "            Self::{}(m) => m.value_range(),", variant)?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Add iter() method that collects all variants from sub-enums
    writeln!(f, "    /// Iterate over all morph targets across all categories")?;
    writeln!(f, "    pub fn iter() -> impl Iterator<Item = Self> {{")?;
    writeln!(f, "        use strum::IntoEnumIterator;")?;
    writeln!(f, "        std::iter::empty()")?;
    for category in &category_names {
        let entries = entries_by_category.get(category).unwrap();
        if entries.is_empty() {
            continue;
        }
        let variant = sanitize_name(category);
        let enum_name = format!("{}Morph", variant);
        writeln!(f, "            .chain({}::iter().map(Self::{}))", enum_name, variant)?;
    }
    writeln!(f, "    }}")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

/// Extract base name by removing pair-type suffix and l-/r- prefix
fn extract_base_name(target_name: &str, pair_type: PairType) -> String {
    let mut name = target_name.to_string();

    // Remove l-/r- prefix (will add back as suffix)
    let is_left = name.starts_with("l-");
    let is_right = name.starts_with("r-");
    if is_left || is_right {
        name = name[2..].to_string();
    }

    // Remove pair-type specific suffixes
    let suffixes = match pair_type {
        PairType::IncrDecr => vec!["-incr", "-decr"],
        PairType::UpDown => vec!["-up", "-down"],
        PairType::InOut => vec!["-in", "-out"],
        PairType::ForwardBackward => vec!["-forward", "-backward"],
        PairType::ConvexConcave => vec!["-convex", "-concave"],
        PairType::CompressUncompress => vec!["-compress", "-uncompress"],
        PairType::SquareRound => vec!["-square", "-round"],
        PairType::PointedTriangle => vec!["-pointed", "-triangle"],
        PairType::Single => vec![],
    };

    for suffix in suffixes {
        if name.ends_with(suffix) {
            name = name[..name.len() - suffix.len()].to_string();
            break;
        }
    }

    name
}

/// Expression target names from assets/make_human/targets/expression/units/
/// These are per-ethnicity (caucasian/african/asian)
const EXPRESSION_TARGETS: &[&str] = &[
    "eye-left-closure",
    "eye-left-opened-up",
    "eye-left-slit",
    "eye-right-closure",
    "eye-right-opened-up",
    "eye-right-slit",
    "eyebrows-left-down",
    "eyebrows-left-extern-up",
    "eyebrows-left-inner-up",
    "eyebrows-left-up",
    "eyebrows-right-down",
    "eyebrows-right-extern-up",
    "eyebrows-right-inner-up",
    "eyebrows-right-up",
    "mouth-compression",
    "mouth-corner-puller",
    "mouth-depression",
    "mouth-depression-retraction",
    "mouth-elevation",
    "mouth-eversion",
    "mouth-open",
    "mouth-parling",
    "mouth-part-later",
    "mouth-protusion",
    "mouth-pursing",
    "mouth-retraction",
    "mouth-upward-retraction",
    "neck-platysma",
    "nose-compression",
    "nose-depression",
    "nose-left-dilatation",
    "nose-left-elevation",
    "nose-right-dilatation",
    "nose-right-elevation",
];

/// Generate Expression enum for facial animation targets
fn generate_expression_enum(f: &mut File, assets_dir: &Path) -> io::Result<()> {
    // Verify the targets exist
    let expr_dir = assets_dir.join("targets/expression/units/caucasian");
    if !expr_dir.exists() {
        println!("cargo:warning=Expression targets directory not found: {:?}", expr_dir);
        return Ok(());
    }

    writeln!(f, "/// Expression targets for facial animation")?;
    writeln!(f, "/// All values are 0.0 to 1.0 (single-sided morphs)")?;
    writeln!(f, "/// Use with Race to get ethnicity-specific target path")?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, Display, Reflect)]")?;
    writeln!(f, "pub enum Expression {{")?;

    for target in EXPRESSION_TARGETS {
        let variant = sanitize_name(target);
        writeln!(f, "    /// {} (0.0 to 1.0)", target)?;
        writeln!(f, "    {},", variant)?;
    }

    writeln!(f, "}}")?;
    writeln!(f)?;

    // Impl block
    writeln!(f, "impl Expression {{")?;

    // Get target name
    writeln!(f, "    /// Get the target file name (without path or extension)")?;
    writeln!(f, "    pub fn target_name(&self) -> &'static str {{")?;
    writeln!(f, "        match self {{")?;
    for target in EXPRESSION_TARGETS {
        let variant = sanitize_name(target);
        writeln!(f, "            Self::{} => \"{}\",", variant, target)?;
    }
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Get target path for a specific race
    writeln!(f, "    /// Get the full asset path for this expression target")?;
    writeln!(f, "    /// Uses the specified race for ethnicity-specific morphs")?;
    writeln!(f, "    pub fn target_path(&self, race: Race) -> String {{")?;
    writeln!(f, "        format!(\"make_human/targets/expression/units/{{}}/{{}}.target\", race.as_str(), self.target_name())")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Get target path with default caucasian
    writeln!(f, "    /// Get the asset path using caucasian variant (default)")?;
    writeln!(f, "    pub fn default_path(&self) -> &'static str {{")?;
    writeln!(f, "        match self {{")?;
    for target in EXPRESSION_TARGETS {
        let variant = sanitize_name(target);
        writeln!(f, "            Self::{} => \"make_human/targets/expression/units/caucasian/{}.target\",", variant, target)?;
    }
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}

/// Generate Phenotype struct and Race enum for macrodetails
fn generate_phenotype(f: &mut File, _assets_dir: &Path) -> io::Result<()> {
    // Race enum
    writeln!(f, "/// Race for phenotype macrodetails")?;
    writeln!(f, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, EnumIter, Display, Reflect)]")?;
    writeln!(f, "pub enum Race {{")?;
    writeln!(f, "    #[default]")?;
    writeln!(f, "    Caucasian,")?;
    writeln!(f, "    African,")?;
    writeln!(f, "    Asian,")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    writeln!(f, "impl Race {{")?;
    writeln!(f, "    pub fn as_str(&self) -> &'static str {{")?;
    writeln!(f, "        match self {{")?;
    writeln!(f, "            Self::Caucasian => \"caucasian\",")?;
    writeln!(f, "            Self::African => \"african\",")?;
    writeln!(f, "            Self::Asian => \"asian\",")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    // Phenotype struct with all sliders
    writeln!(f, "/// Phenotype controls for macrodetail morphs")?;
    writeln!(f, "/// All values are 0.0 to 1.0")?;
    writeln!(f, "#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]")?;
    writeln!(f, "#[reflect(Component)]")?;
    writeln!(f, "pub struct Phenotype {{")?;
    writeln!(f, "    /// Race affects base body shape")?;
    writeln!(f, "    pub race: Race,")?;
    writeln!(f, "    /// 0.0 = female, 1.0 = male")?;
    writeln!(f, "    pub gender: f32,")?;
    writeln!(f, "    /// 0.0 = baby, ~0.19 = child, ~0.5 = young, 1.0 = old")?;
    writeln!(f, "    pub age: f32,")?;
    writeln!(f, "    /// 0.0 = min muscle, 0.5 = average, 1.0 = max")?;
    writeln!(f, "    pub muscle: f32,")?;
    writeln!(f, "    /// 0.0 = min weight, 0.5 = average, 1.0 = max")?;
    writeln!(f, "    pub weight: f32,")?;
    writeln!(f, "    /// 0.0 = min height, 1.0 = max height")?;
    writeln!(f, "    pub height: f32,")?;
    writeln!(f, "    /// 0.0 = uncommon, 1.0 = ideal proportions")?;
    writeln!(f, "    pub proportions: f32,")?;
    writeln!(f, "    /// 0.0 = min cup, 0.5 = average, 1.0 = max (female only)")?;
    writeln!(f, "    pub cupsize: f32,")?;
    writeln!(f, "    /// 0.0 = min firmness, 0.5 = average, 1.0 = max (female only)")?;
    writeln!(f, "    pub firmness: f32,")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    writeln!(f, "impl Default for Phenotype {{")?;
    writeln!(f, "    fn default() -> Self {{")?;
    writeln!(f, "        Self {{")?;
    writeln!(f, "            race: Race::Caucasian,")?;
    writeln!(f, "            gender: 0.0,  // female")?;
    writeln!(f, "            age: 0.5,     // young adult")?;
    writeln!(f, "            muscle: 0.5,  // average")?;
    writeln!(f, "            weight: 0.5,  // average")?;
    writeln!(f, "            height: 0.5,  // average")?;
    writeln!(f, "            proportions: 0.75, // somewhat ideal")?;
    writeln!(f, "            cupsize: 0.5, // average")?;
    writeln!(f, "            firmness: 0.5, // average")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    // Helper to interpolate between two values based on position in range
    writeln!(f, "/// Interpolation weight for a value within a range segment")?;
    writeln!(f, "#[derive(Debug, Clone, Copy)]")?;
    writeln!(f, "pub struct InterpWeight {{")?;
    writeln!(f, "    pub low: &'static str,")?;
    writeln!(f, "    pub high: &'static str,")?;
    writeln!(f, "    pub t: f32, // 0.0 = all low, 1.0 = all high")?;
    writeln!(f, "}}")?;
    writeln!(f)?;

    // Generate resolve methods
    writeln!(f, "impl Phenotype {{")?;

    // Gender interpolation
    writeln!(f, "    fn gender_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        InterpWeight {{ low: \"female\", high: \"male\", t: self.gender.clamp(0.0, 1.0) }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Age interpolation (3 segments)
    writeln!(f, "    fn age_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        let v = self.age.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.1875 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"baby\", high: \"child\", t: v / 0.1875 }}")?;
    writeln!(f, "        }} else if v < 0.5 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"child\", high: \"young\", t: (v - 0.1875) / (0.5 - 0.1875) }}")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            InterpWeight {{ low: \"young\", high: \"old\", t: (v - 0.5) / 0.5 }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Muscle interpolation (2 segments)
    writeln!(f, "    fn muscle_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        let v = self.muscle.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.5 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"minmuscle\", high: \"averagemuscle\", t: v / 0.5 }}")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            InterpWeight {{ low: \"averagemuscle\", high: \"maxmuscle\", t: (v - 0.5) / 0.5 }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Weight interpolation (2 segments)
    writeln!(f, "    fn weight_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        let v = self.weight.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.5 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"minweight\", high: \"averageweight\", t: v / 0.5 }}")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            InterpWeight {{ low: \"averageweight\", high: \"maxweight\", t: (v - 0.5) / 0.5 }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Height (only at extremes)
    writeln!(f, "    fn height_weights(&self) -> Option<InterpWeight> {{")?;
    writeln!(f, "        let v = self.height.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.49 {{")?;
    writeln!(f, "            Some(InterpWeight {{ low: \"minheight\", high: \"\", t: 1.0 - v / 0.49 }})")?;
    writeln!(f, "        }} else if v > 0.51 {{")?;
    writeln!(f, "            Some(InterpWeight {{ low: \"\", high: \"maxheight\", t: (v - 0.51) / 0.49 }})")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            None")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Proportions (only at extremes)
    writeln!(f, "    fn proportions_weights(&self) -> Option<InterpWeight> {{")?;
    writeln!(f, "        let v = self.proportions.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.5 {{")?;
    writeln!(f, "            Some(InterpWeight {{ low: \"uncommonproportions\", high: \"\", t: 1.0 - v / 0.5 }})")?;
    writeln!(f, "        }} else if v > 0.5 {{")?;
    writeln!(f, "            Some(InterpWeight {{ low: \"\", high: \"idealproportions\", t: (v - 0.5) / 0.5 }})")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            None")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Cupsize interpolation (2 segments)
    writeln!(f, "    fn cupsize_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        let v = self.cupsize.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.5 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"mincup\", high: \"averagecup\", t: v / 0.5 }}")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            InterpWeight {{ low: \"averagecup\", high: \"maxcup\", t: (v - 0.5) / 0.5 }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Firmness interpolation (2 segments)
    writeln!(f, "    fn firmness_weights(&self) -> InterpWeight {{")?;
    writeln!(f, "        let v = self.firmness.clamp(0.0, 1.0);")?;
    writeln!(f, "        if v < 0.5 {{")?;
    writeln!(f, "            InterpWeight {{ low: \"minfirmness\", high: \"averagefirmness\", t: v / 0.5 }}")?;
    writeln!(f, "        }} else {{")?;
    writeln!(f, "            InterpWeight {{ low: \"averagefirmness\", high: \"maxfirmness\", t: (v - 0.5) / 0.5 }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Resolve to list of (path, weight) for race-gender-age targets
    writeln!(f, "    /// Get weighted race-gender-age target paths")?;
    writeln!(f, "    /// Returns up to 8 targets with interpolation weights")?;
    writeln!(f, "    pub fn race_gender_age_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = Vec::new();")?;
    writeln!(f, "        let gender = self.gender_weights();")?;
    writeln!(f, "        let age = self.age_weights();")?;
    writeln!(f, "        let race = self.race.as_str();")?;
    writeln!(f)?;
    writeln!(f, "        // 2x2 interpolation: gender (low/high) x age (low/high)")?;
    writeln!(f, "        for (g_name, g_weight) in [(gender.low, 1.0 - gender.t), (gender.high, gender.t)] {{")?;
    writeln!(f, "            if g_weight < 0.001 {{ continue; }}")?;
    writeln!(f, "            for (a_name, a_weight) in [(age.low, 1.0 - age.t), (age.high, age.t)] {{")?;
    writeln!(f, "                if a_weight < 0.001 {{ continue; }}")?;
    writeln!(f, "                let path = format!(\"make_human/targets/macrodetails/{{}}-{{}}-{{}}.target\", race, g_name, a_name);")?;
    writeln!(f, "                result.push((path, g_weight * a_weight));")?;
    writeln!(f, "            }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Resolve universal gender-age-muscle-weight targets
    writeln!(f, "    /// Get weighted universal gender-age-muscle-weight target paths")?;
    writeln!(f, "    /// Returns up to 16 targets with interpolation weights")?;
    writeln!(f, "    pub fn universal_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = Vec::new();")?;
    writeln!(f, "        let gender = self.gender_weights();")?;
    writeln!(f, "        let age = self.age_weights();")?;
    writeln!(f, "        let muscle = self.muscle_weights();")?;
    writeln!(f, "        let weight = self.weight_weights();")?;
    writeln!(f)?;
    writeln!(f, "        // 2^4 = 16 corner interpolation")?;
    writeln!(f, "        for (g_name, g_w) in [(gender.low, 1.0 - gender.t), (gender.high, gender.t)] {{")?;
    writeln!(f, "            if g_w < 0.001 {{ continue; }}")?;
    writeln!(f, "            for (a_name, a_w) in [(age.low, 1.0 - age.t), (age.high, age.t)] {{")?;
    writeln!(f, "                if a_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                for (m_name, m_w) in [(muscle.low, 1.0 - muscle.t), (muscle.high, muscle.t)] {{")?;
    writeln!(f, "                    if m_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                    for (w_name, w_w) in [(weight.low, 1.0 - weight.t), (weight.high, weight.t)] {{")?;
    writeln!(f, "                        if w_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                        let path = format!(\"make_human/targets/macrodetails/universal-{{}}-{{}}-{{}}-{{}}.target\", g_name, a_name, m_name, w_name);")?;
    writeln!(f, "                        result.push((path, g_w * a_w * m_w * w_w));")?;
    writeln!(f, "                    }}")?;
    writeln!(f, "                }}")?;
    writeln!(f, "            }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Height targets
    writeln!(f, "    /// Get weighted height target paths")?;
    writeln!(f, "    pub fn height_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = Vec::new();")?;
    writeln!(f, "        let Some(height) = self.height_weights() else {{ return result; }};")?;
    writeln!(f, "        let gender = self.gender_weights();")?;
    writeln!(f, "        let age = self.age_weights();")?;
    writeln!(f, "        let muscle = self.muscle_weights();")?;
    writeln!(f, "        let weight = self.weight_weights();")?;
    writeln!(f)?;
    writeln!(f, "        let h_name = if !height.low.is_empty() {{ height.low }} else {{ height.high }};")?;
    writeln!(f, "        let h_w = if !height.low.is_empty() {{ height.t }} else {{ height.t }};")?;
    writeln!(f, "        if h_w < 0.001 {{ return result; }}")?;
    writeln!(f)?;
    writeln!(f, "        for (g_name, g_w) in [(gender.low, 1.0 - gender.t), (gender.high, gender.t)] {{")?;
    writeln!(f, "            if g_w < 0.001 {{ continue; }}")?;
    writeln!(f, "            for (a_name, a_w) in [(age.low, 1.0 - age.t), (age.high, age.t)] {{")?;
    writeln!(f, "                if a_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                for (m_name, m_w) in [(muscle.low, 1.0 - muscle.t), (muscle.high, muscle.t)] {{")?;
    writeln!(f, "                    if m_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                    for (w_name, w_w) in [(weight.low, 1.0 - weight.t), (weight.high, weight.t)] {{")?;
    writeln!(f, "                        if w_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                        let path = format!(\"make_human/targets/macrodetails/height/{{}}-{{}}-{{}}-{{}}-{{}}.target\", g_name, a_name, m_name, w_name, h_name);")?;
    writeln!(f, "                        result.push((path, g_w * a_w * m_w * w_w * h_w));")?;
    writeln!(f, "                    }}")?;
    writeln!(f, "                }}")?;
    writeln!(f, "            }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Proportions targets
    writeln!(f, "    /// Get weighted proportions target paths")?;
    writeln!(f, "    pub fn proportions_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = Vec::new();")?;
    writeln!(f, "        let Some(props) = self.proportions_weights() else {{ return result; }};")?;
    writeln!(f, "        let gender = self.gender_weights();")?;
    writeln!(f, "        let age = self.age_weights();")?;
    writeln!(f, "        let muscle = self.muscle_weights();")?;
    writeln!(f, "        let weight = self.weight_weights();")?;
    writeln!(f)?;
    writeln!(f, "        let p_name = if !props.low.is_empty() {{ props.low }} else {{ props.high }};")?;
    writeln!(f, "        let p_w = props.t;")?;
    writeln!(f, "        if p_w < 0.001 {{ return result; }}")?;
    writeln!(f)?;
    writeln!(f, "        for (g_name, g_w) in [(gender.low, 1.0 - gender.t), (gender.high, gender.t)] {{")?;
    writeln!(f, "            if g_w < 0.001 {{ continue; }}")?;
    writeln!(f, "            for (a_name, a_w) in [(age.low, 1.0 - age.t), (age.high, age.t)] {{")?;
    writeln!(f, "                if a_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                for (m_name, m_w) in [(muscle.low, 1.0 - muscle.t), (muscle.high, muscle.t)] {{")?;
    writeln!(f, "                    if m_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                    for (w_name, w_w) in [(weight.low, 1.0 - weight.t), (weight.high, weight.t)] {{")?;
    writeln!(f, "                        if w_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                        let path = format!(\"make_human/targets/macrodetails/proportions/{{}}-{{}}-{{}}-{{}}-{{}}.target\", g_name, a_name, m_name, w_name, p_name);")?;
    writeln!(f, "                        result.push((path, g_w * a_w * m_w * w_w * p_w));")?;
    writeln!(f, "                    }}")?;
    writeln!(f, "                }}")?;
    writeln!(f, "            }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // Cupsize/firmness targets (female only, in breast folder)
    // NOTE: Files only exist when cup OR firmness is NOT average (sparse grid)
    writeln!(f, "    /// Get weighted cupsize/firmness target paths (female only)")?;
    writeln!(f, "    pub fn breast_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = Vec::new();")?;
    writeln!(f, "        // Only applies to female (gender < 0.5)")?;
    writeln!(f, "        if self.gender > 0.5 {{ return result; }}")?;
    writeln!(f, "        let gender_factor = 1.0 - self.gender * 2.0; // 1.0 at female, 0.0 at 0.5")?;
    writeln!(f)?;
    writeln!(f, "        let age = self.age_weights();")?;
    writeln!(f, "        let muscle = self.muscle_weights();")?;
    writeln!(f, "        let weight = self.weight_weights();")?;
    writeln!(f, "        let cupsize = self.cupsize_weights();")?;
    writeln!(f, "        let firmness = self.firmness_weights();")?;
    writeln!(f)?;
    writeln!(f, "        for (a_name, a_w) in [(age.low, 1.0 - age.t), (age.high, age.t)] {{")?;
    writeln!(f, "            if a_w < 0.001 {{ continue; }}")?;
    writeln!(f, "            for (m_name, m_w) in [(muscle.low, 1.0 - muscle.t), (muscle.high, muscle.t)] {{")?;
    writeln!(f, "                if m_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                for (w_name, w_w) in [(weight.low, 1.0 - weight.t), (weight.high, weight.t)] {{")?;
    writeln!(f, "                    if w_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                    for (c_name, c_w) in [(cupsize.low, 1.0 - cupsize.t), (cupsize.high, cupsize.t)] {{")?;
    writeln!(f, "                        if c_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                        for (f_name, f_w) in [(firmness.low, 1.0 - firmness.t), (firmness.high, firmness.t)] {{")?;
    writeln!(f, "                            if f_w < 0.001 {{ continue; }}")?;
    writeln!(f, "                            // Skip averagecup-averagefirmness (no file exists)")?;
    writeln!(f, "                            if c_name == \"averagecup\" && f_name == \"averagefirmness\" {{ continue; }}")?;
    writeln!(f, "                            let path = format!(\"make_human/targets/breast/female-{{}}-{{}}-{{}}-{{}}-{{}}.target\", a_name, m_name, w_name, c_name, f_name);")?;
    writeln!(f, "                            result.push((path, a_w * m_w * w_w * c_w * f_w * gender_factor));")?;
    writeln!(f, "                        }}")?;
    writeln!(f, "                    }}")?;
    writeln!(f, "                }}")?;
    writeln!(f, "            }}")?;
    writeln!(f, "        }}")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;
    writeln!(f)?;

    // All targets combined
    writeln!(f, "    /// Get all phenotype target paths with weights")?;
    writeln!(f, "    pub fn all_targets(&self) -> Vec<(String, f32)> {{")?;
    writeln!(f, "        let mut result = self.race_gender_age_targets();")?;
    writeln!(f, "        result.extend(self.universal_targets());")?;
    writeln!(f, "        result.extend(self.height_targets());")?;
    writeln!(f, "        result.extend(self.proportions_targets());")?;
    writeln!(f, "        result.extend(self.breast_targets());")?;
    writeln!(f, "        result")?;
    writeln!(f, "    }}")?;

    writeln!(f, "}}")?;
    writeln!(f)?;

    Ok(())
}
