# Architecture

The map of the code: what lives where, and the words we use for things.
**Agents and humans read this before writing code, and update it in the same change when they add, move or remove a public item.**

The big picture (data flow, crate rules) lives in `TECH_STACK.md` §2 and the rules in `AGENTS.md`. This file is the index.

---

## Code map

Status: `planned` = not written yet, `done` = exists. Keep the "Key items" column to the public functions and types others should reuse.

### `crates/chess_core`: rules and game state (no Bevy)
| Module | Owns | Key items | Status |
|---|---|---|---|
| `lib.rs` | Crate root, re-exports. `cozy-chess` stays private behind it | — | done |
| `types` | Board vocabulary | `Square` (`new`, `from_index`, `file`, `rank`, parse/print `"e4"`), `Color` (`opponent`), `PieceKind`, `Piece` | done |
| `command` | Move requests from any input source | `MoveCommand`, `CommandSource`, `MoveError` | done |
| `game` | Position, legal moves, applying commands, result incl. all draw rules | `Game` (`new`, `from_fen`, `apply`, `legal_moves`, `piece_at`, `pieces`, `side_to_move`, `in_check`, `result`), `Move` (with `capture` and `rook_move` for animation), `GameResult` | done |
| `notation` | SAN and PGN reading/writing (beta: save/load) | — | planned |

Tests: `crates/chess_core/tests/rules.rs` covers perft (start, Kiwipete, en passant pins), special moves, rejections and every game result.

### `crates/chess_ai`: computer opponent (no Bevy)
| Module | Owns | Key items | Status |
|---|---|---|---|
| `lib.rs` | Negamax alpha-beta search, capture-first move ordering, evaluation (material + center/advance bonuses). Deterministic | `choose_move(game, depth)` | done |

Tests: `crates/chess_ai/tests/search.rs` (mate in one, free material, promotion, game over, determinism + depth timing). Split into `search`/`eval` modules only when the file grows.

### `crates/game`: Bevy app
| Module | Owns | Key items | Status |
|---|---|---|---|
| `main.rs` | App setup, logging setup, plugin registration | — | done |
| `board` | Board tiles and base slab, scene lighting, square ↔ world position, selection highlights | `BoardPlugin`, `square_to_world`, `world_to_square`, `SQUARE_SIZE` | done |
| `pieces` | Piece entities (re)spawned from `ActiveGame` on `GameStarted` / `ThemeReady`; theme model or placeholder shape, each army facing the other | `PiecesPlugin`, `PieceOnBoard`, `PieceModel`, `PieceSpawner` (`spawn`), `army_facing` | done |
| `input` | Mouse picking (clicked point → square via `world_to_square`, so any model works): select a piece, click a target → `MoveCommandSent`; promotions wait for a `PromotionChoice` click; locked during animations and the AI's turn; R after game over → `NewGameRequested` | `InputPlugin`, `Selection` | done |
| `turn` | Owns the active game, applies `MoveCommandSent` via `chess_core`, announces `MovePlayed`; starts new games (`NewGameRequested` → `GameStarted`); defines the per-frame `GameSet` order | `TurnPlugin`, `ActiveGame`, `MoveCommandSent`, `MovePlayed`, `NewGameRequested`, `GameStarted`, `GameSet` | done |
| `animation` | Animation queue / state machine: walk (facing the path, knights hop) → attack → hit → die → walk on and settle; castling rook, promotion; placeholders skip clip steps and shrink; locks input while busy; cleared on new game | `AnimationPlugin`, `AnimationQueue` (`is_busy`) | done |
| `clips` | Hooks a model's `idle/walk/attack/hit/die` clips to its `AnimationPlayer` (graph cached per model file), starts idle, plays clips with crossfade | `ClipsPlugin`, `Clip`, `PieceAnimator`, `ClipPlayer` (`play`, `has`) | done |
| `ai` | Runs `chess_ai` on `AsyncComputeTaskPool` on the AI's turn, sends its move as a `MoveCommandSent` (source `Ai`); cancels on new game | `AiPlugin`, `AiPlayer` (`white`, `black`, `depth`, `plays`) | done |
| `theme` | Loads `themes/<name>/theme.ron` (`CHESS_THEME`, default placeholder) via a RON asset loader; builds model paths; falls back to placeholders | `ThemePlugin`, `ThemeManifest`, `PieceModels`, `ThemeStatus` (`model_path`), `ThemeReady` | done |
| `camera` | Orbit camera: right-drag rotates, scroll zooms; faces the human's side on game start; background color | `CameraPlugin` | done |
| `audio` | Sound effects and music | — | planned |
| `ui` | Status line (turn, check, AI), promotion picker, game-over banner; shared UI building blocks | `UiPlugin`, `PromotionChoice`, `spawn_button`, `centered_layer`, `set_visible`, `TEXT_COLOR`, `PANEL_BACKGROUND`, `BUTTON_BACKGROUND` | done |
| `menu` | Main menu (opponent, side, AI level, Start), app flow states, Esc back to menu; `CHESS_DEMO=1` dev demo (AI vs AI, no menu) | `MenuPlugin`, `AppState`, `MatchSettings`, `AiLevel` | done |
| `test_support` | Test-only: headless app and `play()` helper shared by all `game` tests | `headless_app`, `play` | done |

---

## Where does it go?

| If you need… | Put it in / use |
|---|---|
| Anything about chess rules or legality | `chess_core`, never `game` |
| A conversion between a `Square` and a 3D position | `game::board::square_to_world` (reuse it, never recompute) |
| A new way for players to give moves | A new producer of `MoveCommand` in `game::input` (or its own crate later) |
| A system that sends, applies or shows moves | Put it in the matching `turn::GameSet` (`Input` → `Turn` → `Animation`) so a move flows through in one frame |
| An asset path | The theme manifest, loaded by `game::theme`; models follow `docs/ASSET_SPEC.md` |
| Spawning a piece's visual (new game, promotion, …) | `pieces::PieceSpawner::spawn`, never build piece entities by hand |
| Playing an animation clip on a piece | `clips::ClipPlayer::play`; new visual sequences are new `animation::Step`s |
| Which way a piece faces | `pieces::army_facing` (resting) |
| A UI button, panel or overlay | Build it from `ui::spawn_button` / `ui::centered_layer` and the shared colors; never copy the node setup |
| A system that should only run during play (or only in the menu) | `.run_if(in_state(AppState::Playing))` |
| A tunable number (speeds, sizes, colors) | A `const` at the top of the module that uses it; move it to settings only when players need to change it |

---

## Glossary

Use exactly these names in code, comments and docs. Don't invent synonyms.

| Term | Meaning | Don't call it |
|---|---|---|
| `Square` | One of the 64 board locations (a1–h8) | tile, cell, field, position |
| File / Rank | Board column a–h / row 1–8; in code `Square::file()` / `Square::rank()`, both 0–7 | column, row, x/y |
| `Color` | White or Black side | team, player, side |
| `PieceKind` | Pawn, Knight, Bishop, Rook, Queen, King | piece type, role |
| Piece (entity) | The 3D character in the Bevy world, with a `PieceOnBoard` component (its `Piece` and `Square`) | unit, figure, troop (in code) |
| `Selection` | The selected square plus its legal moves (`input::Selection`) | picked, chosen, active piece |
| `MoveCommandSent` | Bevy message carrying a `MoveCommand` from any input source to the turn flow | move event, move request |
| `MovePlayed` | Bevy message carrying a `Move` the rules accepted; the animation queue shows it | move event, applied move |
| `GameSet` | The per-frame system order: `Input` → `Turn` → `Animation` | stage, phase |
| `NewGameRequested` / `GameStarted` | Bevy messages: input asks for a new game; the turn flow announces that one began (startup or restart) | restart event, reset |
| AI | The computer opponent (`chess_ai`, `game::ai`, `AiPlayer`); shown to players as "Computer" in the menu | bot, engine, CPU |
| `AppState` | App flow: `Menu` or `Playing` | screen, scene, mode |
| `MatchSettings` / `AiLevel` | The menu's choices (opponent, side, AI level) applied on Start | config, options, difficulty |
| `ThemeStatus` / `ThemeReady` | Which theme is in use (placeholder, loading, ready); message sent when it's ready | skin status |
| `Clip` | One of the five standard model animations: idle, walk, attack, hit, die | animation, action (Blender's word), motion |
| `PieceAnimator` | A piece model's hookup to its animation player and clip nodes | rig, controller |
| `ActiveGame` | Bevy resource holding the `Game` being played; the only source of truth in the `game` crate | game state, board state |
| `Move` | A legal move as `chess_core` represents it | action, turn |
| `MoveCommand` | A *request* to make a move, from any input source; may be rejected | move request, order, input move |
| `Game` | The full game state: position, history, result | board state, match |
| Position | The board arrangement plus side to move, castling and en passant rights | board, FEN (FEN is its text format) |
| Turn | One side's chance to move | round, ply (ply = one move by one side; use only in AI code) |
| Capture | A move that removes an enemy piece | kill, take (fine in voice phrases only) |
| Theme | A swappable set of piece models, board and scenery | skin, style, pack |
| Animation queue | Ordered visual steps that play out an applied move | timeline, sequence |

---

## Reference code

Once a module is reviewed and considered the model to copy, list it here. Agents match its style, structure, comments and logging.

- *(none yet; the first finished alpha module becomes the reference)*
