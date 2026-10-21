#!/usr/bin/env python3
# @file make_test_theme.py
# @brief Generates the dev-only "test" theme: boxes with a nose on their front and the five standard clips.
# @note Dev asset for checking the model pipeline and animations without real art. Not shipped, git-ignored.
#       Usage: python3 scripts/make_test_theme.py [output_dir]   (default: crates/game/assets/themes/test)

import json
import math
import struct
import sys
from pathlib import Path

# (kind, body width, height): taller for more valuable pieces, like the placeholders.
KINDS = [
    ("pawn", 0.40, 0.50),
    ("knight", 0.45, 0.70),
    ("bishop", 0.42, 0.80),
    ("rook", 0.50, 0.65),
    ("queen", 0.48, 0.95),
    ("king", 0.50, 1.05),
]
# Linear color, as glTF stores it: these match the placeholders' sRGB cream (0.93) and dark stone (0.13).
COLORS = {"white": [0.85, 0.79, 0.64, 1.0], "black": [0.016, 0.013, 0.018, 1.0]}


def box(lo, hi):
    """@brief Positions, normals and triangle indices of an axis-aligned box, faces wound outward."""
    (x0, y0, z0), (x1, y1, z1) = lo, hi
    faces = [
        ((1, 0, 0), [(x1, y0, z1), (x1, y0, z0), (x1, y1, z0), (x1, y1, z1)]),
        ((-1, 0, 0), [(x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)]),
        ((0, 1, 0), [(x0, y1, z1), (x1, y1, z1), (x1, y1, z0), (x0, y1, z0)]),
        ((0, -1, 0), [(x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)]),
        ((0, 0, 1), [(x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]),
        ((0, 0, -1), [(x1, y0, z0), (x0, y0, z0), (x0, y1, z0), (x1, y1, z0)]),
    ]
    positions, normals, indices = [], [], []
    for normal, corners in faces:
        # Make sure each face is wound counter-clockwise when seen from outside.
        a, b, c = corners[0], corners[1], corners[2]
        cross = (
            (b[1] - a[1]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[1] - a[1]),
            (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]),
            (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]),
        )
        if sum(n * k for n, k in zip(normal, cross)) < 0:
            corners = corners[::-1]
        start = len(positions)
        positions += corners
        normals += [normal] * 4
        indices += [start, start + 1, start + 2, start, start + 2, start + 3]
    return positions, normals, indices


def build_glb(width, height, color):
    """@brief A complete .glb: body box plus nose on the +Z (front) side, with idle/walk/attack/hit/die clips."""
    body = box((-width / 2, 0, -width / 2), (width / 2, height, width / 2))
    nose = box((-0.08, height * 0.6, width / 2), (0.08, height * 0.75, width / 2 + 0.12))
    offset = len(body[0])
    positions = body[0] + nose[0]
    normals = body[1] + nose[1]
    indices = body[2] + [i + offset for i in nose[2]]

    blob, views, accessors = bytearray(), [], []

    def add(data, fmt, count, kind, component, target=None, minmax=None):
        """@brief Appends packed data as a buffer view plus accessor; returns the accessor index."""
        while len(blob) % 4:
            blob.append(0)
        packed = struct.pack("<" + fmt * (len(data) // len(fmt)), *data)
        view = {"buffer": 0, "byteOffset": len(blob), "byteLength": len(packed)}
        if target:
            view["target"] = target
        blob.extend(packed)
        views.append(view)
        accessor = {"bufferView": len(views) - 1, "componentType": component, "count": count, "type": kind}
        if minmax:
            accessor["min"], accessor["max"] = minmax
        accessors.append(accessor)
        return len(accessors) - 1

    flat = lambda rows: [v for row in rows for v in row]
    mins = [min(p[i] for p in positions) for i in range(3)]
    maxs = [max(p[i] for p in positions) for i in range(3)]
    pos = add(flat(positions), "f", len(positions), "VEC3", 5126, 34962, (mins, maxs))
    nrm = add(flat(normals), "f", len(normals), "VEC3", 5126, 34962)
    idx = add(indices, "H", len(indices), "SCALAR", 5123, 34963)

    half_tilt = 0.15  # half-angle of the hit tilt, radians
    tilt = [math.sin(half_tilt), 0.0, 0.0, math.cos(half_tilt)]
    clips = {
        "idle": ("translation", [0, 1, 2], [[0, 0, 0], [0, 0.03, 0], [0, 0, 0]]),
        "walk": ("translation", [0, 0.25, 0.5], [[0, 0, 0], [0, 0.08, 0], [0, 0, 0]]),
        "attack": ("translation", [0, 0.25, 0.6], [[0, 0, 0], [0, 0.05, 0.3], [0, 0, 0]]),
        "hit": ("rotation", [0, 0.15, 0.4], [[0, 0, 0, 1], tilt, [0, 0, 0, 1]]),
        "die": ("scale", [0, 0.3, 0.8], [[1, 1, 1], [1.1, 0.3, 1.1], [0.01, 0.01, 0.01]]),
    }
    animations = []
    for name, (path, times, values) in clips.items():
        kind = "VEC4" if path == "rotation" else "VEC3"
        inp = add(times, "f", len(times), "SCALAR", 5126, minmax=([min(times)], [max(times)]))
        out = add(flat(values), "f", len(values), kind, 5126)
        animations.append({
            "name": name,
            "samplers": [{"input": inp, "output": out, "interpolation": "LINEAR"}],
            "channels": [{"sampler": 0, "target": {"node": 1, "path": path}}],
        })

    gltf = {
        "asset": {"version": "2.0", "generator": "3d_chess make_test_theme.py"},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"name": "piece", "children": [1]}, {"name": "body", "mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": pos, "NORMAL": nrm}, "indices": idx, "material": 0}]}],
        "materials": [{"pbrMetallicRoughness": {"baseColorFactor": color, "metallicFactor": 0.0, "roughnessFactor": 0.6}}],
        "animations": animations,
        "buffers": [{"byteLength": len(blob)}],
        "bufferViews": views,
        "accessors": accessors,
    }
    while len(blob) % 4:
        blob.append(0)
    text = json.dumps(gltf, separators=(",", ":")).encode()
    text += b" " * (-len(text) % 4)
    total = 12 + 8 + len(text) + 8 + len(blob)
    return (
        struct.pack("<III", 0x46546C67, 2, total)
        + struct.pack("<II", len(text), 0x4E4F534A) + text
        + struct.pack("<II", len(blob), 0x004E4942) + bytes(blob)
    )


def main():
    """@brief Writes every piece for both colors plus the theme manifest."""
    out = Path(sys.argv[1] if len(sys.argv) > 1 else "crates/game/assets/themes/test")
    manifest = ["(", '    name: "Test (dev)",']
    for side, color in COLORS.items():
        (out / side).mkdir(parents=True, exist_ok=True)
        entries = []
        for kind, width, height in KINDS:
            (out / side / f"{kind}.glb").write_bytes(build_glb(width, height, color))
            entries.append(f'{kind}: Some("{side}/{kind}.glb")')
        manifest.append(f"    {side}: ({', '.join(entries)}),")
    manifest.append(")")
    (out / "theme.ron").write_text("\n".join(manifest) + "\n")
    print(f"test theme written to {out}")


if __name__ == "__main__":
    main()
