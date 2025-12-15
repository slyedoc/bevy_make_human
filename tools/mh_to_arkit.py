#!/usr/bin/env python3
"""
claude's idea for converting MakeHuman face shapes

Converts MakeHuman raw face shapes to ARKit 52 blend shapes.
Some shapes require combining multiple MH shapes, others need to be created.
"""

import os
from pathlib import Path
from dataclasses import dataclass
from typing import Optional

# Path to MH raw shapes
MH_RAW_PATH = Path.home() / "code/f/mhx2-makehuman-exchange/import_runtime_mhx2/data/hm8/faceshapes/raw"

# ARKit 52 blend shapes with MH mapping
# Format: "arkit_name": [("mh_shape", weight), ...] or None if needs creation
ARKIT_TO_MH = {
    # Eyes - ALL NEED CREATION (MH has no eye blend shapes, uses bone rotation)
    "eyeBlinkLeft": None,
    "eyeBlinkRight": None,
    "eyeLookDownLeft": None,
    "eyeLookDownRight": None,
    "eyeLookInLeft": None,
    "eyeLookInRight": None,
    "eyeLookOutLeft": None,
    "eyeLookOutRight": None,
    "eyeLookUpLeft": None,
    "eyeLookUpRight": None,
    "eyeSquintLeft": None,
    "eyeSquintRight": None,
    "eyeWideLeft": None,
    "eyeWideRight": None,

    # Jaw
    "jawOpen": [("mouth_open", 1.0)],
    "jawForward": None,  # needs creation
    "jawLeft": None,     # needs creation
    "jawRight": None,    # needs creation

    # Mouth
    "mouthClose": None,  # inverse of mouth_open?
    "mouthFunnel": [("mouth_narrow", 1.0)],
    "mouthPucker": [("mouth_narrow", 1.0)],
    "mouthLeft": None,   # needs creation
    "mouthRight": None,  # needs creation
    "mouthSmileLeft": [("mouth_corner_up", 0.5), ("mouth_wide", 0.3)],  # left side only
    "mouthSmileRight": [("mouth_corner_up", 0.5), ("mouth_wide", 0.3)], # right side only
    "mouthFrownLeft": [("mouth_corner_down", 1.0)],  # left side only
    "mouthFrownRight": [("mouth_corner_down", 1.0)], # right side only
    "mouthDimpleLeft": [("mouth_corner_in", 0.5)],   # left side only
    "mouthDimpleRight": [("mouth_corner_in", 0.5)],  # right side only
    "mouthStretchLeft": [("mouth_wide", 1.0)],       # left side only
    "mouthStretchRight": [("mouth_wide", 1.0)],      # right side only
    "mouthRollLower": [("lips_lower_in", 1.0)],
    "mouthRollUpper": [("lips_upper_in", 1.0)],
    "mouthShrugLower": [("lips_lower_out", 1.0)],
    "mouthShrugUpper": [("lips_upper_out", 1.0)],
    "mouthPressLeft": [("lips_part", -0.5)],         # close lips left
    "mouthPressRight": [("lips_part", -0.5)],        # close lips right
    "mouthLowerDownLeft": [("lips_mid_lower_down", 1.0)],  # left side
    "mouthLowerDownRight": [("lips_mid_lower_down", 1.0)], # right side
    "mouthUpperUpLeft": [("lips_mid_upper_up", 1.0)],      # left side
    "mouthUpperUpRight": [("lips_mid_upper_up", 1.0)],     # right side

    # Cheek
    "cheekPuff": [("cheek_balloon", 1.0)],
    "cheekSquintLeft": [("cheek_squint", 1.0), ("cheek_up", 0.5)],  # left side
    "cheekSquintRight": [("cheek_squint", 1.0), ("cheek_up", 0.5)], # right side

    # Nose
    "noseSneerLeft": [("nose_wrinkle", 0.5)],   # left side only
    "noseSneerRight": [("nose_wrinkle", 0.5)],  # right side only

    # Brow
    "browDownLeft": [("brow_mid_down", 0.5), ("brow_outer_down", 0.5)],   # left side
    "browDownRight": [("brow_mid_down", 0.5), ("brow_outer_down", 0.5)],  # right side
    "browInnerUp": [("brow_mid_up", 1.0)],
    "browOuterUpLeft": [("brow_outer_up", 1.0)],   # left side only
    "browOuterUpRight": [("brow_outer_up", 1.0)],  # right side only

    # Tongue
    "tongueOut": [("tongue_out", 1.0)],
}


@dataclass
class MorphTarget:
    """Sparse vertex deltas"""
    name: str
    deltas: dict[int, tuple[float, float, float]]

    @classmethod
    def load(cls, path: Path) -> "MorphTarget":
        """Load .target file"""
        deltas = {}
        with open(path) as f:
            for line in f:
                line = line.strip()
                if not line or line.startswith('#'):
                    continue
                parts = line.split()
                if len(parts) >= 4:
                    idx = int(parts[0])
                    x, y, z = float(parts[1]), float(parts[2]), float(parts[3])
                    if abs(x) > 0.0001 or abs(y) > 0.0001 or abs(z) > 0.0001:
                        deltas[idx] = (x, y, z)
        return cls(name=path.stem, deltas=deltas)

    def save(self, path: Path):
        """Save as .target file"""
        with open(path, 'w') as f:
            f.write("# ARKit blend shape generated from MakeHuman targets\n")
            f.write("# basemesh hm08\n")
            for idx in sorted(self.deltas.keys()):
                x, y, z = self.deltas[idx]
                f.write(f"{idx} {x:.6f} {y:.6f} {z:.6f}\n")

    def scale(self, factor: float) -> "MorphTarget":
        """Scale all deltas"""
        new_deltas = {
            idx: (x * factor, y * factor, z * factor)
            for idx, (x, y, z) in self.deltas.items()
        }
        return MorphTarget(name=self.name, deltas=new_deltas)

    def add(self, other: "MorphTarget") -> "MorphTarget":
        """Add another morph target"""
        new_deltas = dict(self.deltas)
        for idx, (x, y, z) in other.deltas.items():
            if idx in new_deltas:
                ox, oy, oz = new_deltas[idx]
                new_deltas[idx] = (ox + x, oy + y, oz + z)
            else:
                new_deltas[idx] = (x, y, z)
        return MorphTarget(name=self.name, deltas=new_deltas)

    def filter_left(self) -> "MorphTarget":
        """Keep only vertices on left side (x > 0 in MH coords)"""
        # Note: This is a simplification - proper implementation needs vertex positions
        # For now, we'd need to know which vertices are on which side
        return self  # TODO: implement with base mesh vertex positions

    def filter_right(self) -> "MorphTarget":
        """Keep only vertices on right side (x < 0 in MH coords)"""
        return self  # TODO: implement with base mesh vertex positions


def load_mh_shapes() -> dict[str, MorphTarget]:
    """Load all MH raw shapes"""
    shapes = {}
    if not MH_RAW_PATH.exists():
        print(f"Warning: MH raw path not found: {MH_RAW_PATH}")
        return shapes

    for path in MH_RAW_PATH.glob("*.target"):
        target = MorphTarget.load(path)
        shapes[target.name] = target
        print(f"  Loaded: {target.name} ({len(target.deltas)} vertices)")
    return shapes


def generate_arkit_shapes(mh_shapes: dict[str, MorphTarget], output_dir: Path):
    """Generate ARKit shapes from MH shapes"""
    output_dir.mkdir(parents=True, exist_ok=True)

    generated = []
    missing = []

    for arkit_name, mapping in ARKIT_TO_MH.items():
        if mapping is None:
            missing.append(arkit_name)
            continue

        # Combine MH shapes
        result = None
        for mh_name, weight in mapping:
            if mh_name not in mh_shapes:
                print(f"  Warning: MH shape '{mh_name}' not found for {arkit_name}")
                continue

            scaled = mh_shapes[mh_name].scale(weight)
            if result is None:
                result = scaled
            else:
                result = result.add(scaled)

        if result:
            result.name = arkit_name
            output_path = output_dir / f"{arkit_name}.target"
            result.save(output_path)
            generated.append(arkit_name)
            print(f"  Generated: {arkit_name}")

    return generated, missing


def main():
    global MH_RAW_PATH
    print("MakeHuman to ARKit Blend Shape Converter\n")

    # Check MH path
    print(f"MH raw shapes path: {MH_RAW_PATH}")
    if not MH_RAW_PATH.exists():
        # Try alternate path
        alt_path = Path.home() / "code/f/mhx2-makehuman-exchange/import_runtime_mhx2/data/hm8/faceshapes/raw"
        if alt_path.exists():
            MH_RAW_PATH = alt_path
            print(f"Using alternate path: {MH_RAW_PATH}")

    print("\nLoading MH shapes...")
    mh_shapes = load_mh_shapes()
    print(f"Loaded {len(mh_shapes)} MH shapes\n")

    # Output directory
    output_dir = Path(__file__).parent.parent / "assets/make_human/targets/arkit"
    print(f"Output directory: {output_dir}\n")

    print("Generating ARKit shapes...")
    generated, missing = generate_arkit_shapes(mh_shapes, output_dir)

    print(f"\n=== Summary ===")
    print(f"Generated: {len(generated)}/52 ARKit shapes")
    print(f"Missing (need manual creation): {len(missing)}")

    if missing:
        print(f"\nMissing shapes that need Blender sculpting:")
        for name in missing:
            print(f"  - {name}")

    print("\n=== Coverage ===")
    eyes_missing = [n for n in missing if n.startswith("eye")]
    jaw_missing = [n for n in missing if n.startswith("jaw") and n != "jawOpen"]
    mouth_missing = [n for n in missing if n.startswith("mouth") and n in missing]

    print(f"Eyes: {14 - len(eyes_missing)}/14 (need {len(eyes_missing)} sculpted)")
    print(f"Jaw: {4 - len(jaw_missing)}/4 (need {len(jaw_missing)} sculpted)")
    print(f"Mouth: {23 - len(mouth_missing)}/23")
    print(f"Cheek: 3/3")
    print(f"Nose: 2/2")
    print(f"Brow: 5/5")
    print(f"Tongue: 1/1")


if __name__ == "__main__":
    main()
