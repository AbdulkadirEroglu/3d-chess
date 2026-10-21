<div align="center">

<img src="docs/assets/banner.svg" alt="3D Chess: every piece is a soldier, every capture is a fight" width="100%">

<br>

![Status](https://img.shields.io/badge/status-pre--alpha-d4a24c?style=for-the-badge)
![Rust](https://img.shields.io/badge/rust-1.98-ce422b?style=for-the-badge&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/bevy-0.20-232326?style=for-the-badge&logo=bevy&logoColor=white)
[![CI](https://img.shields.io/github/actions/workflow/status/AbdulkadirEroglu/3d-chess/ci.yml?branch=master&style=for-the-badge&label=CI&logo=githubactions&logoColor=white)](https://github.com/AbdulkadirEroglu/3d-chess/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a7bd5?style=for-the-badge)](#-license)

![Linux](https://img.shields.io/badge/Linux-first-FCC624?style=flat-square&logo=linux&logoColor=black)
![Steam](https://img.shields.io/badge/Steam-planned-1b2838?style=flat-square&logo=steam&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-planned-0078D6?style=flat-square&logo=windows&logoColor=white)
![Android](https://img.shields.io/badge/Android-planned-3DDC84?style=flat-square&logo=android&logoColor=white)

**[Features](#-features)** · **[Roadmap](#-roadmap)** · **[Quick start](#-quick-start)** · **[How it works](#-how-it-works)** · **[Contributing](#-contributing)**

</div>

---

## ♞ What is this?

> Real chess rules, but the pieces are **animated 3D troops**. They march to their squares, clash when they capture, and shatter when they fall. Command them with the mouse or, later, **with your voice**.

---

## ✨ Features

<table>
<tr>
<td width="33%" valign="top">

### 🏰 Living pieces
Stone soldiers that walk, gallop and slide. Every move is animated.

`alpha` · `beta`

</td>
<td width="33%" valign="top">

### ⚔️ Real fights
Captures are battles: the attacker strikes, the defender breaks apart.

`beta`

</td>
<td width="33%" valign="top">

### 🤖 AI opponent
No one to play with? The built-in AI takes the other side.

`alpha`

</td>
</tr>
<tr>
<td valign="top">

### 🎙️ Voice commands
*"Knight to F3."* Push to talk and your troops obey. Works offline.

`after v1.0`

</td>
<td valign="top">

### 🎨 Themes
Swap the army, board and scenery. Stone comes first; more follow.

`after v1.0`

</td>
<td valign="top">

### 🌐 Multiplayer
Hot-seat on one machine now, online matches later.

`alpha` · `after v1.0`

</td>
</tr>
</table>

---

## 🗺️ Roadmap

```mermaid
flowchart LR
    A["🧪 v0.1<br/>Alpha<br/>Linux · core loop · AI"]:::current
    B["🧩 v0.5<br/>Beta<br/>stone theme · GitHub release"]:::planned
    C["🐧 v1.0<br/>Linux release<br/>polish · own rules engine"]:::planned
    D["🎮 v1.x<br/>Steam + Windows<br/>Deck · gamepad"]:::planned
    E["📱 v2.x<br/>Android<br/>touch · mobile tiers"]:::planned
    F["🍎 v3.x<br/>iOS<br/>if the game takes off"]:::maybe
    A --> B --> C --> D --> E -.-> F

    classDef current fill:#d4a24c,stroke:#8a6420,color:#15141a,stroke-width:2px
    classDef planned fill:#2a2630,stroke:#5d5750,color:#f3ead8
    classDef maybe fill:#2a2630,stroke:#5d5750,color:#8f8799,stroke-dasharray:4 3
```

| | Milestone | Highlights |
|:-:|---|---|
| 🟡 | **v0.1 Alpha** | Full rules · mouse controls · placeholder pieces · local 2-player · AI opponent |
| ⚪ | **v0.5 Beta** | Stone army · animated captures · menus & sound · public Linux builds |
| ⚪ | **v1.0 Linux** | Polish · cinematic captures · our own rules engine · AppImage/Flatpak |
| ⚪ | **v1.x Steam** | Windows · Steam Deck · achievements · gamepad |
| ⚪ | **v2.x Android** | Touch controls · mobile performance |
| ⚪ | **v3.x iOS** | Only if the game finds its players |

<sub>🟡 in progress · ⚪ planned · Full details and feature tracks in <a href="ROADMAP.md">ROADMAP.md</a></sub>

---

## 🚀 Quick start

```sh
git clone https://github.com/AbdulkadirEroglu/3d-chess.git && cd 3d-chess
cargo run -p game
```

<details>
<summary><b>📦 System libraries (Linux)</b></summary>

<br>

| Distro | Command |
|---|---|
| Arch | `sudo pacman -S alsa-lib systemd-libs wayland libxkbcommon` |
| Debian / Ubuntu | `sudo apt install libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev` |
| Fedora | `sudo dnf install alsa-lib-devel systemd-devel wayland-devel libxkbcommon-devel` |

The Rust toolchain is pinned in `rust-toolchain.toml`; `rustup` installs the right version automatically.

</details>

<details>
<summary><b>⚡ Developer commands</b></summary>

<br>

| Task | Command |
|---|---|
| **Run all CI checks locally** | `./scripts/ci.sh` |
| Run (fast rebuilds) | `cargo run -p game --features bevy/dynamic_linking` |
| Watch the AI play itself (animation check, soak test) | `CHESS_DEMO=1 cargo run -p game` |
| Play with a theme pack | `CHESS_THEME=test cargo run -p game` (generate it first: `python3 scripts/make_test_theme.py`) |
| Check models against the spec | `python3 scripts/check_asset.py crates/game/assets/themes/<name>` |
| Verbose logs | `RUST_LOG=game=debug,chess_ai=trace cargo run -p game` |
| Test | `cargo test --workspace` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Format | `cargo fmt --all` |
| Licenses & advisories | `cargo deny check` |
| Unused dependencies | `cargo machete` |
| Enable pre-commit checks | `git config core.hooksPath .githooks` |

</details>

---

## 🧠 How it works

Every input (mouse, voice, AI or network) becomes the same **`MoveCommand`**. The rules decide; the 3D world just shows what happened.

```mermaid
flowchart LR
    subgraph IN["🎮 Input sources"]
        M[🖱️ Mouse / touch]
        V[🎙️ Voice]
        AI[🤖 AI]
        N[🌐 Network]
    end
    MC{{"MoveCommand"}}
    R["♟️ chess_core<br/>validates & applies"]
    Q["🎬 Animation queue<br/>walk · strike · shatter"]
    X["❌ Rejected<br/>feedback to source"]
    M & V & AI & N --> MC --> R
    R -- legal --> Q
    R -- illegal --> X

    classDef core fill:#d4a24c,stroke:#8a6420,color:#15141a
    class MC,R core
```

<details>
<summary><b>📁 Project layout</b></summary>

<br>

```
3d-chess/
├── crates/
│   ├── chess_core/   ♟️  rules & game state (plain Rust, no engine)
│   ├── chess_ai/     🤖 computer opponent
│   └── game/         🎮 Bevy app: board, pieces, input, animation, UI
├── assets/themes/    🎨 swappable theme packs
├── docs/             📚 design notes & decisions
├── .claude/          🧩 agent settings, hooks & skills
└── .githooks/        ✅ pre-commit checks
```

</details>

<details>
<summary><b>🧰 Tech stack</b></summary>

<br>

| Layer | Choice |
|---|---|
| Language | ![Rust](https://img.shields.io/badge/Rust_2024-ce422b?logo=rust&logoColor=white) |
| Engine | ![Bevy](https://img.shields.io/badge/Bevy_0.20-232326?logo=bevy&logoColor=white) |
| Chess rules | `cozy-chess` → our own engine before v1.0 |
| Physics / Audio | `avian3d` · `bevy_kira_audio` |
| Voice (planned) | Whisper (desktop) · Vosk (mobile) |
| Assets | Blender → glTF · quality tiers per platform |

Full rationale, versions and licenses: [TECH_STACK.md](TECH_STACK.md)

</details>

---

## 🤝 Contributing

| 📖 Read first | |
|---|---|
| [`AGENTS.md`](AGENTS.md) | Rules for humans and AI agents: simplicity, comments, logging, scope |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Code map & glossary: **check here before writing anything new** |
| [`TECH_STACK.md`](TECH_STACK.md) | Approved crates, versions, license policy |
| [`ROADMAP.md`](ROADMAP.md) | What's being built now, and what comes later |

Working with Claude Code? Use **`/feature <task>`** to build and **`/audit`** to review before committing.

---

## 📜 License

Free and open source. The code is dual-licensed under either of

| | |
|---|---|
| [MIT License](LICENSE-MIT) | short and simple |
| [Apache License 2.0](LICENSE-APACHE) | adds an explicit patent grant |

at your option. This is the same as Bevy and most of the Rust ecosystem.

Unless you explicitly state otherwise, any contribution you intentionally submit for inclusion in this project shall be dual-licensed as above, without any additional terms or conditions.

<sub>Game assets (models, textures, audio) may carry their own licenses, listed in <code>assets/CREDITS.md</code> once assets arrive.</sub>

---

<div align="center">

<sub>Built with 🦀 Rust and 🕊️ Bevy</sub>

</div>
