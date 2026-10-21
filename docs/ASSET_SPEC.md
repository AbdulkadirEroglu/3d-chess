# Asset spec: piece models

What a piece model must look like for the game to load, place and animate it.
Check every export with the validator before trying it in game:

```sh
python3 scripts/check_asset.py crates/game/assets/themes/stone            # whole theme
python3 scripts/check_asset.py crates/game/assets/themes/stone/white/knight.glb
```

---

## 1. Theme layout

```
crates/game/assets/themes/<theme>/
├── theme.ron          # manifest: name + one model file per piece and color
├── white/
│   ├── pawn.glb  knight.glb  bishop.glb  rook.glb  queen.glb  king.glb
└── black/
    └── pawn.glb  knight.glb  bishop.glb  rook.glb  queen.glb  king.glb
```

**One file per piece per color.** Each army gets its own materials, so export each piece twice (or keep two Blender files).

`theme.ron`:
```ron
(
    name: "Stone",
    white: (
        pawn: Some("white/pawn.glb"),
        knight: Some("white/knight.glb"),
        bishop: Some("white/bishop.glb"),
        rook: Some("white/rook.glb"),
        queen: Some("white/queen.glb"),
        king: Some("white/king.glb"),
    ),
    black: (
        pawn: Some("black/pawn.glb"),
        // … same for the other pieces
    ),
)
```
A piece left out uses its placeholder shape, so a theme can be built up one piece at a time.

Run a theme with `CHESS_THEME=stone cargo run -p game`.

## 2. Size, origin and facing

| Rule | Value | Why |
|---|---|---|
| Units | 1 Blender unit = 1 m = **one board square** | The board is 8 × 8 units |
| Footprint | Fits inside **x, z ∈ [-0.5, 0.5]**; aim for ≤ 0.8 wide | Pieces must not overlap neighbors; clicks are mapped by position |
| Origin | **Center of the base, at floor level** (lowest point y = 0) | The game places the origin on the square's surface |
| Facing | **Looks at you in Blender's Front view** (Numpad 1) | Exported as +Z; the game turns each army toward its opponent |
| Height guide | pawn ~0.5, knight ~0.7, bishop ~0.8, rook ~0.65, queen ~0.95, king ~1.05 | Readable silhouettes; taller = more valuable |
| Max height | 1.4 | Keeps the camera framing sane |

Apply all transforms before export (`Ctrl+A` → All Transforms), so scale is 1 and rotation is 0.

## 3. Mesh, rig and materials

| Item | Rule |
|---|---|
| Meshes | One skinned mesh per piece (small extra rigid parts are fine if parented to bones) |
| Rig | One armature, root bone at the origin |
| Materials | PBR metallic-roughness (Principled BSDF); textures embedded in the `.glb`. Colors are exported in linear space; Blender handles that, so pick colors as they look in Blender |
| Texture sizes | Author at 4096², export per tier (below) |

### Quality tiers ("author high, export down")

| Tier | Triangles per piece | Textures | Use |
|---|---|---|---|
| `high` | ≤ 50k | 2048²–4096² | Desktop, high settings |
| `medium` | ≤ 20k | 1024²–2048² | Desktop low, Steam Deck |
| `low` | ≤ 8k | 512²–1024² | Mobile (later) |

Each tier is its own clean game mesh with the sculpt's detail baked into normal maps. **Don't decimate a rigged mesh to make a lower tier.** Beta ships `high` first; check with `--tier high|medium|low`.

## 4. Animations

Five clips, named **exactly** like this (they're the Blender action names):

| Clip | Loops | Length | What it shows |
|---|:-:|---|---|
| `idle` | yes | 2–4 s | Breathing, small shifts; the piece's resting pose |
| `walk` | yes | 0.5–1 s per cycle | Moving to another square (knights may hop instead) |
| `attack` | no | 0.5–1 s | Striking forward (+Z) at an enemy one square ahead |
| `hit` | no | 0.3–0.6 s | Taking a blow |
| `die` | no | 0.6–1.5 s | Falling / crumbling; ends on the ground |

- 30 fps. Loops must start and end on the same pose.
- Root motion: **don't** move the root in `walk` or `attack`; the game moves the piece across the board. Animate the body, not the position.
- Extra clips are allowed if prefixed with the piece, e.g. `knight_jump`.

## 5. Export settings (Blender glTF exporter)

| Setting | Value |
|---|---|
| Format | glTF Binary (`.glb`) |
| Include | Selected objects (mesh + armature) |
| Transform | **+Y Up** (default) |
| Data → Mesh | Apply modifiers, UVs, normals, tangents (for normal maps) |
| Animation | On; mode **Actions** (each action becomes a clip with the action's name) |
| Compression | Off for now (Draco isn't loaded by the game) |

## 6. Licensing

Every asset needs a line in `crates/game/assets/CREDITS.md`: file, author, source, license.
For AI-generated meshes or textures, record the tool and its model license. Some forbid commercial use or restrict regions, and Steam requires disclosing AI-generated content.

## 7. Test theme

`python3 scripts/make_test_theme.py` generates `themes/test`: boxes with a "nose" on their front, and all five clips as simple motions. It's a dev-only asset (git-ignored) for checking the pipeline: `CHESS_THEME=test cargo run -p game`.
