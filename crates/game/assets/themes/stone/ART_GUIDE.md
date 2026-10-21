# Stone theme: art guide

**Start here for the art work.** This guide is self-contained: it's written for a separate chat session (human or AI assistant) that hasn't seen the code discussions. Everything needed to model, rig, animate, export and check the stone pieces is here or linked.

---

## 1. Context

- **The game:** an animated 3D chess game ("living troops" chess, our own IP; never use names or designs from existing franchises). Written in Rust with the Bevy engine. Free, open source, released on GitHub and later Steam (Linux first, then Windows, Android).
- **Repo:** `/home/ae/Work/home_ws/3d_chess` (GitHub: `AbdulkadirEroglu/3d-chess`).
- **Where we are:** v0.5 beta, at the **🛑 art checkpoint** in `ROADMAP.md`. Code development is paused until the stone pieces exist. The game can already load, place and animate models: every piece exported here shows up in game right away.
- **The look:** pieces are **stone soldiers**: weathered, carved, heavy. White army: pale cream stone. Black army: dark stone. On a capture, the attacker walks up, strikes, and the defender takes the hit and crumbles.
- **Hardware:** the work machine has an RTX 5080 for local AI tools (3D generation, texturing).

## 2. What to deliver

12 models: 6 pieces × 2 colors, each with 5 animation clips.

| Piece | White file | Black file | Height guide |
|---|---|---|---|
| Pawn | `white/pawn.glb` | `black/pawn.glb` | ~0.5 |
| Knight | `white/knight.glb` | `black/knight.glb` | ~0.7 |
| Bishop | `white/bishop.glb` | `black/bishop.glb` | ~0.8 |
| Rook | `white/rook.glb` | `black/rook.glb` | ~0.65 |
| Queen | `white/queen.glb` | `black/queen.glb` | ~0.95 |
| King | `white/king.glb` | `black/king.glb` | ~1.05 |

**One file per piece per color**: same mesh and rig, different materials. Export each piece twice.

Animation clips, named **exactly** (Blender action names):

| Clip | Loops | Length | Shows |
|---|:-:|---|---|
| `idle` | yes | 2–4 s | Resting pose, breathing, small shifts |
| `walk` | yes | 0.5–1 s per cycle | Moving between squares (knight may hop) |
| `attack` | no | 0.5–1 s | Striking forward at an enemy one square ahead |
| `hit` | no | 0.3–0.6 s | Taking a blow |
| `die` | no | 0.6–1.5 s | Falling / crumbling; ends on the ground |

The game plays them in this order during a capture: attacker `walk` → `attack` → defender `hit` → `die` → attacker `walk` → `idle`. It times `attack` at 0.6 s, `hit` at 0.35 s, `die` at 0.9 s, so aim near those.

## 3. The hard rules (full spec: `/home/ae/Work/home_ws/3d_chess/docs/ASSET_SPEC.md`)

| Rule | Value |
|---|---|
| Scale | 1 Blender unit = 1 m = **one board square** |
| Footprint | Inside x, z ∈ [-0.5, 0.5]; aim ≤ 0.8 wide |
| Origin | Center of the base at floor level (lowest point y = 0) |
| Facing | **Looks at you in Blender's Front view (Numpad 1)** |
| Transforms | Applied before export (`Ctrl+A` → All Transforms) |
| Mesh / rig | One skinned mesh, one armature, root bone at the origin |
| Root motion | None: don't move the root in `walk`/`attack`; the game moves the piece |
| Materials | Principled BSDF (PBR metallic-roughness), textures embedded |
| Triangles | `high` tier ≤ 50k per piece (start here); `medium` ≤ 20k; `low` ≤ 8k later |
| Textures | Author at 4096², export 2048² for `high` |
| Detail | Sculpt high-poly → retopologized game mesh → bake normal maps. **Never decimate a rigged mesh** to make a lower tier |

**Blender glTF export:** glTF Binary (`.glb`), selected objects (mesh + armature), +Y Up, apply modifiers, include UVs/normals/tangents, animation **On** with mode **Actions**, compression **Off**.

## 4. Where files go

| What | Where | In git? |
|---|---|---|
| Exported game models (`.glb`) | `crates/game/assets/themes/stone/white/` and `black/` (this folder) | Yes, via **Git LFS** |
| Theme manifest | `crates/game/assets/themes/stone/theme.ron` | Yes |
| Source files: `.blend`, sculpts, 4K textures, AI-tool outputs, references | **Outside the repo**, suggested `~/Work/home_ws/3d_chess_art/` | No (too large for the code repo) |
| Credits | `crates/game/assets/CREDITS.md` | Yes |

`docs/assets/` is **not** for models: it only holds images for the documentation.

## 5. Workflow per piece (do the **pawn first**, end to end)

1. **Base form** in Blender at real scale (Section 3), facing Front view.
2. **Detail and texture**: local AI tools for detail and stone texturing, then clean up in Blender. Keep all raw outputs in the source folder.
3. **Game mesh**: retopologize to the `high` budget; bake normal/AO from the high-poly.
4. **Rig**: one armature, root bone at the origin; weight paint.
5. **Animate** the five clips as Blender actions named exactly `idle`, `walk`, `attack`, `hit`, `die`.
6. **Export** `white/<piece>.glb`, switch materials, export `black/<piece>.glb`.
7. **Validate** (Section 6); fix until it passes.
8. **Enable** it in `theme.ron`: uncomment its two lines.
9. **Check in game** (Section 6): size, facing, materials, and all clips during moves and captures.
10. **Credit** it in `crates/game/assets/CREDITS.md` (Section 8) and tick it in Section 9.

The pawn surfaces every pipeline question (scale, facing, rig, export). The other five go much faster after it.

## 6. Commands (run from `/home/ae/Work/home_ws/3d_chess`)

```sh
# Check one model, or the whole theme, against the spec
python3 scripts/check_asset.py crates/game/assets/themes/stone/white/pawn.glb
python3 scripts/check_asset.py crates/game/assets/themes/stone

# See it in game: AI plays both sides, so every animation shows up without clicking
CHESS_THEME=stone CHESS_DEMO=1 cargo run -p game

# Play it yourself with the stone theme
CHESS_THEME=stone cargo run -p game

# Reference: the generated test theme shows what a passing model looks like
python3 scripts/make_test_theme.py
CHESS_THEME=test CHESS_DEMO=1 cargo run -p game
```

The validator checks clip names, triangle budget, footprint, origin height and rig count. While working through the theme, "missing" errors for pieces not made yet are expected. In game, right-drag rotates the camera and the scroll wheel zooms.

## 7. Before the first `.glb` is committed: Git LFS

Git LFS isn't installed yet. Once:

```sh
sudo pacman -S git-lfs            # Omarchy / Arch
cd /home/ae/Work/home_ws/3d_chess
git lfs install
git lfs track "*.glb"             # creates .gitattributes; commit it before the first model
```

## 8. Licensing and credits

- Every asset gets a row in `crates/game/assets/CREDITS.md`: file, author, source/tool, license.
- **AI tools:** record the tool and its **model license**. Some forbid commercial use or certain regions; Steam requires disclosing AI-generated content on the store page.
- Our own art is planned as **CC-BY-4.0** (code is MIT / Apache-2.0).
- **Mixamo** or other library animations: their terms may forbid redistributing the raw files in a public repo. Check before using them.

## 9. Progress

| Piece | Modeled | Textured | Rigged | idle | walk | attack | hit | die | White exported | Black exported | Validated | In game |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| Pawn | | | | | | | | | | | | |
| Knight | | | | | | | | | | | | |
| Bishop | | | | | | | | | | | | |
| Rook | | | | | | | | | | | | |
| Queen | | | | | | | | | | | | |
| King | | | | | | | | | | | | |

## 10. Rules for an AI assistant in the art session

- **Scope:** art only. Work in this folder, `theme.ron`, `CREDITS.md`, and the source folder outside the repo. **Don't change code** (`crates/*/src`, scripts, docs other than this guide's progress table). Code questions go back to the main development session.
- **Never commit or push.** The maintainer makes all commits. If something really has to be set aside, use `git stash push -m "<what and why>"` and say so clearly.
- **Ask before installing software** (Blender, AI tools, Git LFS).
- **Simplest working solution first**, and note anything non-obvious (export settings that mattered, tool settings) in the progress table or a short note below it.
- If the spec seems wrong for the art (a size, timing or rule), **say so instead of working around it**. The spec can change, but through the main session.

## 11. Done when (art checkpoint exit)

- All 12 models pass `python3 scripts/check_asset.py crates/game/assets/themes/stone`.
- All entries in `theme.ron` are enabled.
- They look right in `CHESS_THEME=stone CHESS_DEMO=1 cargo run -p game`: scale, facing, both armies, every clip during moves and captures.
- Every asset is in `CREDITS.md`.

Then the main session makes `stone` the default theme and continues with beta step 3.
