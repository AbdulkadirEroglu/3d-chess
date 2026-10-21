# AGENTS.md

Instructions for AI coding agents (and humans) working on this repository. Read this first, every session.

## Project

Animated 3D chess in Rust + Bevy: the pieces are living troops that walk, fight and shatter. It's free and open source, released on GitHub and Steam (Linux first, then Windows, Android, and maybe iOS).

Before working, read:
- `ROADMAP.md`: milestones and which one is current. **Only work on the current milestone's scope.**
- `TECH_STACK.md`: the approved crates, versions, layout and policies.
- `ARCHITECTURE.md`: **the code map and glossary.** What exists, where it lives, and what things are called.
- `docs/adr/`: past architecture decisions, if any.

## Working principles

- **Simplest solution first.** Always look for the most basic solution that correctly solves the problem. Don't add abstractions, generics, traits, config options or "future-proofing" that the current task doesn't need. Complexity has to justify itself; when in doubt, leave it out.
- **One function over many helpers.** If something can be solved clearly in a single function, write a single function. Don't split it into helper functions, wrapper types or extra structs just for structure. Extract a helper only when the same logic is genuinely reused, or the function becomes hard to follow.
- **Document as you go.** Comments are simple, short and easy to understand (see *Comments* below). When behavior, architecture or workflow changes, update the matching doc (`ROADMAP.md`, `TECH_STACK.md`, `AGENTS.md`, `docs/`) in the same change.
- **Always log.** Every feature logs what it does (see *Logging* below).
- **Never commit.** The maintainer makes all commits. Don't run `git commit`, `git push`, `git rebase`, `git merge` or anything that rewrites history. If changes really have to be set aside, use `git stash push -m "<what and why>"` and tell the maintainer clearly that there are stashed changes waiting to be reviewed and committed.

## Before and after writing code

**Before:**
1. Read `ARCHITECTURE.md`.
2. **Search before you write.** For every function, type or constant you're about to add, search the codebase (`rg`) for an existing one. Reuse or extend it. Never create a second function that does the same thing somewhere else.
3. Use the glossary names in `ARCHITECTURE.md`. No synonyms.
4. Match the style of the reference code listed in `ARCHITECTURE.md`.
5. For anything touching more than 3 files or adding a module: write a short plan first and get it approved.

**After:**
1. Update `ARCHITECTURE.md` for every public item you added, moved or removed.
2. In your final report, list every **new public item** (name + file) so duplicates are easy to spot.
3. Touch only what the task needs: no drive-by refactors, renames or reformatting of unrelated code, and no new files unless needed.

## Commands

```sh
cargo build                          # dev build
cargo run -p game                    # run the game
cargo run -p game --features dev     # with debug tooling
cargo test --workspace               # all tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
cargo deny check                     # licenses + advisories
cargo machete                        # unused dependencies
npx --yes jscpd@4 crates --format rust --min-lines 8 --gitignore   # copy-paste detection
cargo run -p game --features bevy/dynamic_linking                  # faster dev rebuilds (never for release)
```

One-time setup per clone: `git config core.hooksPath .githooks` (runs fmt, clippy, tests and deny before each commit).

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

## Comments (Doxygen style)

Use Doxygen tags inside Rust's normal doc comments, `//!` for files and `///` for items, so `cargo doc` still works. Keep them short: one line for `@brief`, a few words per tag.

File header, at the top of every source file:
```rust
//! @file board.rs
//! @brief Spawns the 3D board and highlights squares.
```

Functions, structs, enums and their fields:
```rust
/// @brief Checks a move and applies it to the game.
/// @param game Current game state.
/// @param cmd Move requested by a player, the AI or the network.
/// @return The applied move, or why it was rejected.
pub fn apply_command(game: &mut Game, cmd: MoveCommand) -> Result<Move, MoveError> {
```

- Every function gets a `@brief`, private ones too. Add `@param` / `@return` when they aren't obvious from the name. Use `@note` for a non-obvious *why*, and `@warning` for traps.
- Inline `//` comments go on lines that need explaining, not on every line. They describe *why*, not *what*.

## Logging

Use the `tracing` macros: `error!`, `warn!`, `info!`, `debug!`, `trace!`. In the `game` crate they come from `bevy::log`; in `chess_core` and `chess_ai` they come from the `tracing` crate directly.

| Level | Use for | Examples |
|---|---|---|
| `error!` | Something failed and the game can't do what was asked | Theme pack failed to load, save file corrupt |
| `warn!` | Unexpected but recovered | Missing animation, fallback used; illegal move rejected |
| `info!` | Important, infrequent events | Game started or ended, move played, mode or theme changed, AI chose a move |
| `debug!` | **Simple and repeated events.** Hidden by default, so normal output stays clean | Piece selected, legal-move highlights updated, animation step started or finished, input received |
| `trace!` | Very detailed internals | AI search per depth, per-system internals |

- Log with context: `info!(?cmd, "move played")`, not `info!("move played")`.
- **Never log per frame**, and never inside hot loops (move generation, AI search nodes). Log once around them instead, e.g. "search done: depth 6, 120k nodes, 85 ms".
- Default filter: `info` in release and `debug` in dev builds; override with `RUST_LOG` (e.g. `RUST_LOG=game=debug,chess_ai=trace`).

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
- No `unsafe` (the `unsafe_code` lint denies it). If it's ever truly needed, write an ADR first, then use a local `#[expect(unsafe_code, reason = "...")]` plus a `// SAFETY:` comment.
- Lints are configured in the root `Cargo.toml`. **Never silence a lint to make the build pass.** Fix the code. If a lint is genuinely wrong for a spot, use `#[expect(lint, reason = "why")]` (with a reason, never a bare `#[allow]`).
- Prefer clear code over clever code.
- Don't leave dead code, commented-out code or `TODO`s without an issue or roadmap reference.

## Dependencies

- **Only use crates listed in `TECH_STACK.md`.** To add or upgrade one, update `TECH_STACK.md` in the same change with the reason, the version and the license.
- Add dependencies to `[workspace.dependencies]` in the root `Cargo.toml`, and reference them with `workspace = true`.
- **Never add GPL, AGPL or LGPL crates** (they conflict with Steamworks and the App Store). `cargo deny check` must stay green.
- Never upgrade Bevy as part of feature work.

## Testing

- Any change to `chess_core` comes with tests. Move generation must keep passing the perft suite and, once our own move generator exists, the differential tests against `cozy-chess`.
- Outside `chess_core`, never use `cozy-chess` types directly; it will be replaced before v1.0.
- AI changes: keep search deterministic for a given position and settings, so tests can rely on it. Run the `criterion` benchmarks when touching search performance and report the before/after numbers.
- Bugs get a regression test that fails before the fix.
- Test `game` crate logic (turn flow, command handling) with a headless `App`. Don't test rendering.

## Assets

- glTF binary (`.glb`), one per piece type per theme per quality tier, with the fixed animation names `idle`, `walk`, `attack`, `hit`, `die`.
- Author high, export down: follow the tier budgets in `TECH_STACK.md` §6. Code must never assume one specific tier is present beyond `high`.
- Every new asset gets a line in `assets/CREDITS.md`: source, author, license. Never commit assets with unclear or non-redistributable licenses (e.g. raw Mixamo files).
- Binary assets go through Git LFS.

## Scope and workflow

- Stay inside the current milestone. If you notice a good idea outside it, add it to `ROADMAP.md` (the backlog or the right track) instead of building it.
- Use the project skills: `/feature <task>` to implement something with the full workflow, and `/audit` to review changes before the maintainer commits.
- Make small, focused changes, one concern at a time, so the maintainer can review and commit them easily.
- When you finish, suggest a commit message (imperative mood, optional scope prefix, e.g. `core: add threefold repetition`), but don't commit.
- Significant architectural decisions get a short ADR in `docs/adr/NNNN-title.md`: context, decision, consequences.
- If a request conflicts with these rules or the tech stack, stop and point out the conflict. Don't work around it.

## Don'ts

- Don't edit `Cargo.lock` by hand, or delete it.
- Don't use the `dynamic_linking` feature in release builds.
- Don't commit generated files, build output or large binaries outside LFS.
- Don't introduce the name, logos or other IP of existing franchises (e.g. "Wizard's Chess", Harry Potter) anywhere in code, assets or text.
