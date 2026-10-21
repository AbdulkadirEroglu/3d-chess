# Tech Stack

The technical choices for the project up to v1.0, plus what's deliberately left open for later.
Versions were checked against crates.io on 2026-10-21.

Changing anything here is a decision, not a side effect: update this file in the same change and say why.

---

## 1. Core decisions

| Area | Choice | Version | Notes |
|---|---|---|---|
| Language | Rust, edition 2024 | toolchain `1.98` | Pinned in `rust-toolchain.toml` |
| Engine | Bevy | `0.20` | Released 2026-10-08. MSRV 1.97.1 |
| Chess rules | `cozy-chess` | `0.3` | MIT. Fast bitboard move gen, made for engines |
| Code license | `MIT OR Apache-2.0` | — | Same as Bevy |
| Asset license | CC-BY-4.0 for our own assets (proposed) | — | Third-party assets listed in `assets/CREDITS.md` |

### Why `cozy-chess` and not `shakmaty`
`shakmaty` is more feature-complete (SAN, PGN, variants), but it is **GPL-3.0**, which would make the whole game GPL. GPL conflicts with:
- the **Steamworks SDK** (proprietary, linked into the game), and
- the **Apple App Store** terms (iOS milestone).

`cozy-chess` covers legal move generation, check/mate detection, castling, en passant, promotion and Zobrist hashing. We write the small missing pieces ourselves in `chess_core`: SAN, PGN, and draw rules (50-move, threefold, insufficient material).

### License policy
- **No GPL, AGPL or LGPL dependencies.** `cargo-deny` enforces this in CI.
- **Stockfish** (GPL) may only be used as a **separate executable over UCI**, desktop only, and optional. It is never linked and never required.
- Every asset has a recorded source and license. **Mixamo**: its terms forbid redistributing the raw animation files, so they must not sit in the public repo as-is. Decide how to handle this before beta: re-export through our own rigs, or use other sources.

---

## 2. Workspace layout

A Cargo workspace. The rules and AI crates **do not depend on Bevy**. That keeps them testable, reusable later for online move validation, and unaffected by Bevy upgrades.

```
3d_chess/
├── Cargo.toml              # workspace, shared [workspace.dependencies]
├── rust-toolchain.toml
├── deny.toml               # license + advisory policy
├── crates/
│   ├── chess_core/         # rules, game state, MoveCommand, SAN/PGN, draw rules. No Bevy.
│   ├── chess_ai/           # search engine. Depends on chess_core only. No Bevy.
│   └── game/               # Bevy app (binary). All presentation lives here.
│       └── src/
│           ├── main.rs
│           ├── board/      # board mesh, squares, highlights
│           ├── pieces/     # piece entities, spawning from theme
│           ├── input/      # mouse/keyboard/touch → MoveCommand
│           ├── turn/       # turn flow, applies commands to chess_core
│           ├── animation/  # move/capture sequencing (animation queue)
│           ├── ai/         # bridges chess_ai onto a background task
│           ├── theme/      # theme pack loading
│           ├── camera/
│           ├── audio/
│           └── ui/
├── assets/
│   └── themes/
│       └── placeholder/    # alpha; `stone/` arrives in beta
└── docs/                   # design notes, ADRs (docs/adr/NNNN-title.md)
```

Planned later crates: `voice` (the `SpeechRecognizer` trait and its backends) and `net` (online transport). They plug in as further sources of `MoveCommand`.

### The data flow that everything follows

```
 Mouse / Touch / Voice / AI / Network
                │
                ▼
          MoveCommand { from, to, promotion, source }
                │
                ▼
     turn: validate with chess_core  ──► rejected → feedback to that source
                │ accepted
                ▼
     chess_core state updated (the truth)
                │
                ▼
     animation queue plays the move (a view; can be skipped or sped up)
                │
                ▼
     input unlocked for next side
```

---

## 3. Crates by milestone

### Alpha (v0.1)
| Purpose | Crate | Version | License |
|---|---|---|---|
| Engine | `bevy` | `0.20` | MIT/Apache |
| Chess rules | `cozy-chess` | `0.3` | MIT |
| Picking (click on squares and pieces) | Bevy built-in `bevy_picking` / `MeshPickingPlugin` | — | — |
| Input mapping (mouse, keys, later gamepad and touch) | `leafwing-input-manager` | `0.22` | MIT/Apache |
| Errors (library crates) | `thiserror` | `2` | MIT/Apache |
| Errors (binaries and tools only) | `anyhow` | `1` | MIT/Apache |
| Logging | Bevy's `tracing` integration (`bevy::log`) | — | — |
| Config / manifests | `serde` + `ron` | `1` / `0.12` | MIT/Apache |
| Dev UI (debug panels, inspector) | `bevy_egui` | `0.43` | MIT |

Player-facing UI uses **Bevy UI**. `bevy_egui` is for debug and dev tooling only, behind a `dev` feature.

### Beta to v1.0
| Purpose | Crate | Version | Notes |
|---|---|---|---|
| Audio (music crossfade, SFX control) | `bevy_kira_audio` | `0.27` | Replaces Bevy's built-in audio |
| Physics (capture debris) | `avian3d` | `0.8` | Visual only; never affects game state |
| Particles (dust, sparks) | `bevy_hanabi` | — | Currently on Bevy 0.19; adopt once it supports 0.20 |

### Later milestones (decided when we get there)
| Track | Candidates | Status |
|---|---|---|
| Voice, desktop | `whisper-rs` `0.16` (Unlicense; whisper.cpp MIT) | Preferred |
| Voice, mobile | `vosk` `0.3` (grammar-restricted) vs Whisper | Evaluate on a real device |
| Steam | `steamworks` `0.13` | Behind the `steam` feature |
| Online | Steam networking, `bevy_matchbox` (currently Bevy 0.18), or a custom WebSocket server | **Open.** Write an ADR at the multiplayer milestone |
| Stockfish | External process over UCI | Optional, desktop only |

---

## 4. Testing and quality

| Tool | Use |
|---|---|
| `cargo test` | Unit and integration tests |
| Perft suite | Move-gen correctness through `chess_core` (standard positions with known node counts) |
| `proptest` `1` | Property tests: e.g. "any legal move can be applied and undone", SAN round-trip |
| `criterion` `0.8` | AI search benchmarks (nodes/sec, time to depth) |
| `cargo clippy -- -D warnings` | Lints; warnings fail CI |
| `cargo fmt --check` | Formatting |
| `cargo-deny` | License allow-list, security advisories, duplicate crates |

Rule: `chess_core` and `chess_ai` carry the bulk of the tests. The `game` crate gets tests for turn flow and command handling using a headless Bevy `App` (no window).

## 5. Build, CI, packaging

- **Fast dev builds:** Bevy `dynamic_linking` feature in dev only (never in release builds), the `mold` linker on Linux, and `opt-level = 1` for our code with `opt-level = 3` for dependencies in the dev profile.
- **Release profile:** `lto = "thin"`, `codegen-units = 1`, `strip = true`.
- **CI:** GitHub Actions running fmt, clippy, test, deny and a Linux release build on every PR. Add a Windows job at the Steam milestone and an Android job at the Android milestone.
- **Linux packaging (v1.0):** AppImage and/or Flatpak. Steam uses its own depot.
- **Cargo features:** `dev` (egui tooling), and later `steam`, `voice-whisper`, `voice-vosk`. All off by default in release builds unless a milestone says otherwise.

## 6. Assets pipeline

- **Authoring:** Blender. Meshes may start from AI 3D-gen tools, but get cleaned up and retopologized before rigging.
- **Format:** glTF 2.0 binary (`.glb`). One file per piece type per theme.
- **Animation names (fixed contract):** `idle`, `walk`, `attack`, `hit`, `die`. Optional extras are prefixed per piece (e.g. `knight_jump`).
- **Budgets (mobile-aware):** ≤ 15k triangles per piece, 1024² textures by default (2048² max on desktop), at most 1 skinned mesh per piece.
- **Textures:** PNG while developing; KTX2/Basis compression before Android.
- **Theme pack:** `assets/themes/<name>/theme.ron` manifest plus the `.glb` and texture files. The game never hardcodes asset paths outside the theme loader.
- **Repository:** binary assets go in **Git LFS** from the first real asset. Watch GitHub's LFS bandwidth quota; move source `.blend` files out of the repo if it becomes an issue.

## 7. Platform targets

| Platform | Graphics backend (via wgpu) | Milestone |
|---|---|---|
| Linux x86_64 | Vulkan | Alpha → v1.0 |
| Steam Deck | Vulkan (native Linux build) | Steam |
| Windows x86_64 | DX12 / Vulkan | Steam |
| Android arm64 | Vulkan | v2.x |
| iOS | Metal | v3.x (conditional) |

## 8. Upgrade policy

- Bevy minor upgrades (0.20 → 0.21) happen **between milestones**, as their own change, never mixed with feature work.
- Before upgrading, check that every Bevy plugin we use has a compatible release.
- `Cargo.lock` is committed.
