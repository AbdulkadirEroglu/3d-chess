#!/usr/bin/env python3
# @file check_asset.py
# @brief Checks piece models against docs/ASSET_SPEC.md: clips, triangle budget, size, origin, rig.
# @note Usage: python3 scripts/check_asset.py <file.glb | theme folder> [--tier high|medium|low]
#       Exit code 1 if any model has errors. Warnings don't fail.

import json
import struct
import sys
from pathlib import Path

REQUIRED_CLIPS = {"idle", "walk", "attack", "hit", "die"}
KINDS = ["pawn", "knight", "bishop", "rook", "queen", "king"]
TRIANGLE_BUDGET = {"high": 50_000, "medium": 20_000, "low": 8_000}
MAX_HALF_WIDTH = 0.5  # must fit inside its square
MAX_HEIGHT = 1.4
BASE_TOLERANCE = 0.02  # lowest point must be this close to y = 0
CENTER_TOLERANCE = 0.1  # footprint center this close to the origin


def read_glb(path):
    """@brief Parses a .glb into its JSON document and binary chunk."""
    data = path.read_bytes()
    magic, version, _ = struct.unpack_from("<III", data, 0)
    if magic != 0x46546C67 or version != 2:
        raise ValueError("not a glTF 2.0 binary (.glb) file")
    length, _ = struct.unpack_from("<II", data, 12)
    gltf = json.loads(data[20:20 + length])
    return gltf


def matmul(a, b):
    """@brief Multiplies two 4x4 row-major matrices."""
    return [[sum(a[r][k] * b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def node_matrix(node):
    """@brief Local transform of a glTF node as a 4x4 matrix (from `matrix` or TRS)."""
    if "matrix" in node:
        m = node["matrix"]  # column-major in glTF
        return [[m[c * 4 + r] for c in range(4)] for r in range(4)]
    tx, ty, tz = node.get("translation", [0, 0, 0])
    x, y, z, w = node.get("rotation", [0, 0, 0, 1])
    sx, sy, sz = node.get("scale", [1, 1, 1])
    rot = [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]
    return [
        [rot[0][0] * sx, rot[0][1] * sy, rot[0][2] * sz, tx],
        [rot[1][0] * sx, rot[1][1] * sy, rot[1][2] * sz, ty],
        [rot[2][0] * sx, rot[2][1] * sy, rot[2][2] * sz, tz],
        [0, 0, 0, 1],
    ]


def measure(gltf):
    """@brief World-space bounds and triangle count of the default scene (rest pose)."""
    lo, hi, triangles = [float("inf")] * 3, [float("-inf")] * 3, 0
    identity = [[1 if r == c else 0 for c in range(4)] for r in range(4)]
    scene = gltf.get("scenes", [{}])[gltf.get("scene", 0)]
    stack = [(n, identity) for n in scene.get("nodes", [])]
    while stack:
        index, parent = stack.pop()
        node = gltf["nodes"][index]
        world = matmul(parent, node_matrix(node))
        stack += [(child, world) for child in node.get("children", [])]
        if "mesh" not in node:
            continue
        for prim in gltf["meshes"][node["mesh"]]["primitives"]:
            position = gltf["accessors"][prim["attributes"]["POSITION"]]
            counted = gltf["accessors"][prim["indices"]] if "indices" in prim else position
            if prim.get("mode", 4) == 4:
                triangles += counted["count"] // 3
            pmin, pmax = position.get("min"), position.get("max")
            if pmin is None or pmax is None:
                continue
            for corner in [(x, y, z) for x in (pmin[0], pmax[0]) for y in (pmin[1], pmax[1]) for z in (pmin[2], pmax[2])]:
                point = [sum(world[r][c] * (list(corner) + [1])[c] for c in range(4)) for r in range(3)]
                lo = [min(a, b) for a, b in zip(lo, point)]
                hi = [max(a, b) for a, b in zip(hi, point)]
    return lo, hi, triangles


def check_model(path, tier):
    """@brief Checks one model; returns (errors, warnings) as lists of messages."""
    errors, warnings = [], []
    try:
        gltf = read_glb(path)
    except (OSError, ValueError) as problem:
        return [str(problem)], []

    clips = {a.get("name", "") for a in gltf.get("animations", [])}
    missing = sorted(REQUIRED_CLIPS - clips)
    if missing:
        errors.append(f"missing animation clips: {', '.join(missing)} (found: {', '.join(sorted(clips)) or 'none'})")

    skins = len(gltf.get("skins", []))
    if skins == 0:
        warnings.append("no skin: the mesh isn't rigged (fine for the test theme only)")
    elif skins > 1:
        errors.append(f"{skins} skins; expected one armature")

    lo, hi, triangles = measure(gltf)
    if triangles > TRIANGLE_BUDGET[tier]:
        errors.append(f"{triangles:,} triangles; the {tier} tier allows {TRIANGLE_BUDGET[tier]:,}")
    if lo[0] == float("inf"):
        errors.append("no mesh with position bounds found")
        return errors, warnings
    half_width = max(abs(lo[0]), abs(hi[0]), abs(lo[2]), abs(hi[2]))
    if half_width > MAX_HALF_WIDTH:
        errors.append(f"reaches {half_width:.2f} from the origin; must stay within {MAX_HALF_WIDTH} (one square)")
    if abs(lo[1]) > BASE_TOLERANCE:
        errors.append(f"lowest point is at y = {lo[1]:.3f}; the origin must be at the base (y = 0)")
    if hi[1] - lo[1] > MAX_HEIGHT:
        warnings.append(f"{hi[1] - lo[1]:.2f} tall; the guide maximum is {MAX_HEIGHT}")
    center = ((lo[0] + hi[0]) / 2, (lo[2] + hi[2]) / 2)
    if max(abs(center[0]), abs(center[1])) > CENTER_TOLERANCE:
        warnings.append(f"footprint center is off the origin by ({center[0]:.2f}, {center[1]:.2f})")
    return errors, warnings


def main():
    """@brief Checks a single model or a whole theme folder and prints a report."""
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    tier = "high"
    if "--tier" in sys.argv:
        tier = sys.argv[sys.argv.index("--tier") + 1]
        args = [a for a in args if a != tier]
    if len(args) != 1 or tier not in TRIANGLE_BUDGET:
        print(__doc__ or "usage: check_asset.py <file.glb | theme folder> [--tier high|medium|low]")
        sys.exit(2)

    target = Path(args[0])
    failed = False
    if target.is_dir():
        if not (target / "theme.ron").exists():
            print("ERROR theme.ron: missing")
            failed = True
        models = [target / side / f"{kind}.glb" for side in ("white", "black") for kind in KINDS]
    else:
        models = [target]

    for model in models:
        name = model.relative_to(target) if target.is_dir() else model.name
        if not model.exists():
            print(f"ERROR {name}: missing (the game will use the placeholder)")
            failed = True
            continue
        errors, warnings = check_model(model, tier)
        print(f"{'ERROR' if errors else 'OK   '} {name}")
        for message in errors:
            print(f"      ✗ {message}")
        for message in warnings:
            print(f"      ! {message}")
        failed |= bool(errors)

    print("\nResult:", "FAILED" if failed else f"all models pass ({tier} tier)")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
