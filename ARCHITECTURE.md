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
| `lib.rs` | Crate root, re-exports | — | done |
| `game` | Game state, applying moves, game result | `Game`, `Game::apply`, `GameResult` | planned |
| `command` | Move requests from any input source | `MoveCommand`, `CommandSource`, `MoveError` | planned |
| `notation` | SAN and PGN reading/writing | — | planned |
| `draw` | 50-move, threefold repetition, insufficient material | — | planned |

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
| `board` | Board mesh, square ↔ world position, highlights | — | planned |
| `pieces` | Piece entities, spawning from the theme | — | planned |
| `input` | Mouse/keyboard/touch → `MoveCommand` | — | planned |
| `turn` | Turn flow, sends commands to `chess_core` | — | planned |
| `animation` | Animation queue for moves and captures | — | planned |
| `ai` | Runs `chess_ai` on a background task | — | planned |
| `theme` | Theme pack loading; the only place asset paths live | — | planned |
| `camera` | Camera rig and controls | — | planned |
| `audio` | Sound effects and music | — | planned |
| `ui` | Menus, HUD, game-over screen | — | planned |

---

## Where does it go?

| If you need… | Put it in / use |
|---|---|
| Anything about chess rules or legality | `chess_core`, never `game` |
| A conversion between a `Square` and a 3D position | `game::board` (one function, reuse it) |
| A new way for players to give moves | A new producer of `MoveCommand` in `game::input` (or its own crate later) |
| An asset path | The theme manifest, loaded by `game::theme` |
| A tunable number (speeds, sizes, colors) | A `const` at the top of the module that uses it; move it to settings only when players need to change it |

---

## Glossary

Use exactly these names in code, comments and docs. Don't invent synonyms.

| Term | Meaning | Don't call it |
|---|---|---|
| `Square` | One of the 64 board locations (a1–h8) | tile, cell, field, position |
| `File` / `Rank` | Board column (a–h) / row (1–8) | column, row, x/y |
| `Color` | White or Black side | team, player, side |
| `PieceKind` | Pawn, Knight, Bishop, Rook, Queen, King | piece type, role |
| Piece (entity) | The 3D character in the Bevy world, with a `Piece` component | unit, figure, troop (in code) |
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
