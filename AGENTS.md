# AGENTS.md

Instructions for AI coding agents (and humans) working on this repository. Read this first, every session.

## Project

Animated 3D chess in Rust + Bevy: the pieces are living troops that walk, fight and shatter. It's free and open source, released on GitHub and Steam (Linux first, then Windows, Android, and maybe iOS).

Before working, read:
- `ROADMAP.md`: milestones and which one is current. **Only work on the current milestone's scope.**
- `TECH_STACK.md`: the approved crates, versions, layout and policies.
- `docs/adr/`: past architecture decisions, if any.

## Commands

```sh
cargo build                          # dev build
cargo run -p game                    # run the game
cargo run -p game --features dev     # with debug tooling
cargo test --workspace               # all tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo deny check                     # licenses + advisories
```

Before saying a task is done, run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. All three must pass. If you can't run something, say so explicitly. Don't claim it passed.

## Architecture rules (non-negotiable)

1. **Every move is a `MoveCommand`.** Mouse, touch, voice, AI and network input all produce `MoveCommand` and nothing else. No input path may mutate game state directly.
2. **`chess_core` is the source of truth.** The Bevy scene is a view of it. Never read game state back from entity transforms or animation state.
3. **`chess_core` and `chess_ai` must not depend on Bevy** or on any rendering, audio or platform crate. Keep them plain Rust.
4. **Animation never blocks the rules.** The rules update immediately; the animation queue catches up. Animations must be skippable and speed-adjustable.
5. **Physics and particles are visual only.** They never influence game state.
6. **Assets are loaded through the theme system.** Never hardcode asset paths outside `game/src/theme/`.
7. **The AI runs off the main thread** (Bevy `AsyncComputeTaskPool`) and must never stall a frame.
8. **Platform-specific code** (Steam, voice backends, mobile) sits behind Cargo features and traits, never as `cfg` scattered through gameplay code.

## Bevy conventions

- **Use the Bevy 0.20 API.** Bevy changes a lot between versions, and your training data likely includes older APIs, for example the event/message changes, the bundle-to-required-components move, and picking now being built in. When unsure, check docs.rs for `bevy` 0.20 or the official migration guides. Don't guess.
- One `Plugin` per feature module (`BoardPlugin`, `PiecesPlugin`, `InputPlugin`, ...), registered in `main.rs`.
- Keep systems small and single-purpose. Order them with explicit `SystemSet`s, not implicit ordering.
- Use game states (`States`) for app flow: menu, in-game, game over.
- Components are data; behavior lives in systems. Prefer marker components over boolean flags.
- No `unwrap()`/`expect()` in systems on data that may legitimately be missing; handle it or log it.

## Rust conventions

- Edition 2024, toolchain pinned in `rust-toolchain.toml`.
- Library crates use `thiserror` error types; `anyhow` is only for binaries and tools.
- No `unsafe` without a `// SAFETY:` comment and a reason it's needed.
- Public items in `chess_core` and `chess_ai` get doc comments.
- Prefer clear code over clever code. Comment the *why*, not the *what*.
- Don't leave dead code, commented-out code or `TODO`s without an issue or roadmap reference.

## Dependencies

- **Only use crates listed in `TECH_STACK.md`.** To add or upgrade one, update `TECH_STACK.md` in the same change with the reason, the version and the license.
- Add dependencies to `[workspace.dependencies]` in the root `Cargo.toml`, and reference them with `workspace = true`.
- **Never add GPL, AGPL or LGPL crates** (they conflict with Steamworks and the App Store). `cargo deny check` must stay green.
- Never upgrade Bevy as part of feature work.

## Testing

- Any change to `chess_core` comes with tests. Move generation must keep passing the perft suite.
- AI changes: keep search deterministic for a given position and settings, so tests can rely on it. Run the `criterion` benchmarks when touching search performance and report the before/after numbers.
- Bugs get a regression test that fails before the fix.
- Test `game` crate logic (turn flow, command handling) with a headless `App`. Don't test rendering.

## Assets

- glTF binary (`.glb`), one per piece type per theme, with the fixed animation names `idle`, `walk`, `attack`, `hit`, `die`.
- Budget: ≤ 15k triangles per piece, 1024² textures by default.
- Every new asset gets a line in `assets/CREDITS.md`: source, author, license. Never commit assets with unclear or non-redistributable licenses (e.g. raw Mixamo files).
- Binary assets go through Git LFS.

## Scope and workflow

- Stay inside the current milestone. If you notice a good idea outside it, add it to `ROADMAP.md` (the backlog or the right track) instead of building it.
- Make small, focused changes; one concern per commit or PR.
- Commit messages use the imperative mood, with an optional scope prefix: `core: add threefold repetition`, `game: animate knight jump`.
- Significant architectural decisions get a short ADR in `docs/adr/NNNN-title.md`: context, decision, consequences.
- If a request conflicts with these rules or the tech stack, stop and point out the conflict. Don't work around it.

## Don'ts

- Don't edit `Cargo.lock` by hand, or delete it.
- Don't use the `dynamic_linking` feature in release builds.
- Don't commit generated files, build output or large binaries outside LFS.
- Don't introduce the name, logos or other IP of existing franchises (e.g. "Wizard's Chess", Harry Potter) anywhere in code, assets or text.
