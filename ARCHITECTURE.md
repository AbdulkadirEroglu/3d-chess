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
| `lib.rs` | Crate root, re-exports | — | done |
| `search` | Alpha-beta search, picking the best move | — | planned |
| `eval` | Position evaluation | — | planned |

### `crates/game`: Bevy app
| Module | Owns | Key items | Status |
|---|---|---|---|
| `main.rs` | App setup, logging setup, plugin registration | — | done |
| `board` | Board tiles and base slab, scene lighting, square → world position, selection highlights | `BoardPlugin`, `BoardSquare`, `square_to_world`, `SQUARE_SIZE` | done |
| `pieces` | Piece entities (re)spawned from `ActiveGame` on every `GameStarted`; alpha placeholder shapes (theme models in beta) | `PiecesPlugin`, `PieceOnBoard`, `placeholder_shape` | done |
| `input` | Mouse picking: select a piece, click a target → `MoveCommandSent` (alpha: promotions become a queen); R after game over → `NewGameRequested` | `InputPlugin`, `Selection` | done |
| `turn` | Owns the active game, applies `MoveCommandSent` via `chess_core`, announces `MovePlayed`; starts new games (`NewGameRequested` → `GameStarted`); defines the per-frame `GameSet` order | `TurnPlugin`, `ActiveGame`, `MoveCommandSent`, `MovePlayed`, `NewGameRequested`, `GameStarted`, `GameSet` | done |
| `animation` | Animation queue: slide (knights hop), capture shrink, castling rook, promotion swap; locks input while busy | `AnimationPlugin`, `AnimationQueue` (`is_busy`) | done |
| `ai` | Runs `chess_ai` on a background task | — | planned |
| `theme` | Theme pack loading; the only place asset paths live | — | planned |
| `camera` | Orbit camera: right-drag rotates, scroll zooms; background color | `CameraPlugin` | done |
| `audio` | Sound effects and music | — | planned |
| `ui` | Status line (turn, check) and game-over banner ("Press R to play again"); menus come in step 7 | `UiPlugin` | partial |

---

## Where does it go?

| If you need… | Put it in / use |
|---|---|
| Anything about chess rules or legality | `chess_core`, never `game` |
| A conversion between a `Square` and a 3D position | `game::board::square_to_world` (reuse it, never recompute) |
| A new way for players to give moves | A new producer of `MoveCommand` in `game::input` (or its own crate later) |
| A system that sends, applies or shows moves | Put it in the matching `turn::GameSet` (`Input` → `Turn` → `Animation`) so a move flows through in one frame |
| An asset path | The theme manifest, loaded by `game::theme` |
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
