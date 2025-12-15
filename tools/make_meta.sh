#!/bin/bash
# Create .meta files for any .obj files that don't have them

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ASSETS_DIR="$SCRIPT_DIR/../assets/make_human"

count=0

while read -r obj_file; do
    meta_file="${obj_file}.meta"
    if [ ! -f "$meta_file" ]; then
        cat > "$meta_file" << 'EOF'
(
    meta_format_version: "1.0",
    asset: Load(
        loader: "bevy_make_human::loaders::obj_base_mesh::ObjBaseMeshLoader",
        settings: (),
    ),
)
EOF
        echo "Created: $meta_file"
        ((count++))
    fi
done < <(find "$ASSETS_DIR" -name "*.obj")

echo "Done. Created $count new .meta files."
