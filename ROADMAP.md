# 3D Chess — Roadmap

Animated 3D chess where the pieces are living troops: they walk, fight and shatter.
Written in Rust with Bevy. Free release on GitHub and Steam.

> Working title. Inspired by "wizard chess", but the name, designs and art must be our own.

## Guiding principles

- **Releases are the backbone, features hang off them.** Each milestone has a fixed scope and exit criteria; new ideas go into the backlog, not the current milestone.
- **Every input becomes a `MoveCommand`.** Mouse, touch, voice, AI and network all produce the same command; rules and animation never know the source.
- **Rules are the truth, visuals are a view.** Game state lives in the rules layer (`chess_core`, built on `cozy-chess`); the 3D scene only presents it and can be skipped or sped up.
- **Themes are data.** Pieces, boards and scenery come from swappable theme packs, even before theme switching is exposed to players.
- **Mobile-aware from day one.** Asset budgets and UI layout assume Android will come, even though it ships later.
- **Pinned dependencies.** Bevy is pinned to an exact version and upgraded deliberately between milestones.
- **License-clean.** Code is `MIT OR Apache-2.0`; no GPL-family dependencies (they conflict with Steamworks and the App Store). See `TECH_STACK.md`.

---

## Milestones

### v0.1 — Alpha (Linux, internal)

Goal: the core loop works end to end with placeholder art, playable solo against the AI.

- [ ] Bevy project setup, pinned version, fast-compile config
- [ ] 3D board, orbit camera, lighting
- [ ] Placeholder pieces (simple shapes)
- [ ] Cargo workspace per `TECH_STACK.md`, CI (fmt, clippy, tests, cargo-deny)
- [ ] Full chess rules via `chess_core` (`cozy-chess`): legal moves, check, checkmate, stalemate, castling, en passant, promotion, draw rules
- [ ] Mouse input: select piece, highlight legal moves, choose target
- [ ] `MoveCommand` pipeline: input → rules validation → animation queue
- [ ] Basic move animation (piece travels to target square)
- [ ] Basic capture effect (defender breaks / disappears)
- [ ] Local hot-seat 2-player
- [ ] Simple AI opponent (`chess_ai`: alpha-beta), runs off the main thread
- [ ] Game mode select: human vs human, human vs AI, pick side
- [ ] Game-over screen, restart

**Exit criteria:** a full game can be played start to finish on Linux, vs a human or the AI, with no rule bugs (perft suite passing).

### v0.5 — Beta (Linux, public on GitHub)

Goal: it looks and feels like animated chess, playable by strangers.

- [ ] First theme pack: **stone**
  - [ ] 6 piece models × 2 colors, rigged
  - [ ] Standard animation set: `idle`, `walk`, `attack`, `hit`, `die`
  - [ ] Pre-fractured meshes / debris for captures
- [ ] Animation state machine: walk → attack → defender dies → attacker settles → next turn
- [ ] Piece-specific movement (knight jumps, rook slides, etc.)
- [ ] AI difficulty levels, stronger search (transposition table, move ordering)
- [ ] Main menu, settings (graphics, audio, animation speed)
- [ ] Sound effects and music
- [ ] Undo / move history, save & load game (PGN)
- [ ] Theme pack format defined (folder + manifest + fixed animation names)
- [ ] License chosen for code and assets; asset sources checked
- [ ] GitHub releases with Linux builds, CI

**Exit criteria:** external testers can download, play vs human or AI, and report bugs.

### v1.0 — Full Linux release

Goal: polished, stable, the baseline feature set is complete.

- [ ] Bug fixing and performance pass from beta feedback
- [ ] Capture animations varied per attacker / defender pair (at least per attacker type)
- [ ] Cinematic camera during captures (toggleable)
- [ ] Polished UI, accessibility basics (rebindable keys, scalable UI, colorblind-safe highlights)
- [ ] Stronger AI; optional Stockfish as a separate process over UCI on desktop (never linked, see `TECH_STACK.md`)
- [ ] Packaging: AppImage and/or Flatpak

**Exit criteria:** stable release on GitHub; we'd be happy for anyone to play it.

### v1.x — Windows + Steam release

Goal: reach the main PC audience.

- [ ] Windows build and CI
- [ ] Steam integration via `steamworks`: achievements, cloud saves
- [ ] Steam Deck testing (controller support, Deck-friendly UI)
- [ ] Gamepad support
- [ ] Store page, trailer, screenshots
- [ ] Steam Direct fee and review

**Exit criteria:** live on Steam for Linux and Windows.

### v2.x — Android release

Goal: same game, on phones and tablets.

- [ ] Android build pipeline
- [ ] Touch controls (tap to select / move, pinch & drag camera)
- [ ] Mobile performance pass (LOD, texture sizes, quality presets)
- [ ] UI layout for small screens, portrait and landscape
- [ ] Play Store listing

**Exit criteria:** smooth on a mid-range Android device.

### v3.x — iOS release (conditional)

Only if the game finds an audience and generates income to justify the Apple developer fee and hardware.

- [ ] iOS build pipeline
- [ ] App Store listing

---

## Feature tracks (added along the roadmap)

These are planned after the baseline, each slotted into a milestone once the base is solid. Order is a proposal.

### Voice control (the signature feature)
- [ ] `SpeechRecognizer` trait, platform-independent
- [ ] Push-to-talk
- [ ] Desktop backend: Whisper (`whisper-rs`)
- [ ] Mobile backend: evaluate Vosk (grammar-restricted) vs Whisper
- [ ] Phrase parser: "knight to f3", "queen takes", "castle kingside", optional NATO letters
- [ ] Fuzzy match against current legal moves
- [ ] Voice feedback / confirmation
- [ ] Microphone permissions per platform

### Scenery & backgrounds
- [ ] Environment around the board (hall, field, ruins, …)
- [ ] Lighting presets and day/night
- [ ] Ambient effects (dust, particles)

### Theme switching
- [ ] Theme selection in settings (pieces, board, scenery independently)
- [ ] Additional piece themes beyond stone
- [ ] Possibly community theme packs

### Multiplayer
- [ ] Online play: only moves are sent
- [ ] Transport: Steam networking and/or WebRTC (`matchbox`)
- [ ] Matchmaking / invite links, rematch
- [ ] Clocks and time controls
- [ ] Server-side or peer move validation

### Later ideas (backlog)
- Pieces reacting to threats (trembling pawn, flinching king)
- Persistent debris on the board via physics
- Replays and spectating
- Puzzles / tutorial mode
- Localization

---

## Proposed slotting

| Milestone | Feature tracks |
|---|---|
| v0.1 Alpha | Baseline + simple AI |
| v0.5 Beta | Baseline + stone theme + AI difficulty levels |
| v1.0 Linux | Polish; voice control (desktop) if baseline is stable |
| v1.x Steam | Scenery, theme switching, online multiplayer |
| v2.x Android | Mobile voice backend, touch |
| v3.x iOS | Port only |
