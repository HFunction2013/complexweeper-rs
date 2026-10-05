# Complexweeper

A cross-platform Minesweeper game written in Rust, featuring both **Classic** and **Complex** modes.

## Features

### Classic Mode
- Standard Minesweeper gameplay with one mine type
- Numbered clues (1-8) in classic colors
- Right-click to flag / unflag
- Chord (middle-click or both buttons) to reveal neighbors
- First-click safety guarantee

### Complex Mode
- Four mine types: +Real, -Real, +Imaginary, -Imaginary (+1, -1, +i, -i)
- Clues show the squared magnitude of the complex sum of adjacent mines
- 24 possible display values (integers and simplest radicals)
- Four flag types, cycled with right-click
- Chord criterion: flag count matches mine count, and real/imaginary ratio matches (or is swapped)

### Both Modes
- Three difficulty presets: Beginner (9x9, 10), Intermediate (16x16, 40), Expert (30x16, 99)
- Custom board dimensions and mine counts
- Zoom levels: 100%, 200%, 300%
- Best time tracking per mode and difficulty (stored locally)
- F2 to start a new game
- Classic Windows 95-style visual design

## Building

Requires Rust 1.70+ (with `cargo`).

```bash
cargo build --release
```

The binary will be at `target/release/complexweeper`.

## Running

```bash
cargo run --release
```

## Controls

| Action | Input |
|--------|-------|
| Reveal cell | Left click |
| Flag / cycle flag | Right click |
| Chord (expand neighbors) | Middle click or Left+Right click |
| New game | Click the smiley face or press F2 |
| Menu | Use the menu bar at the top |

## How to Play

### Classic
- Numbers show how many mines are adjacent to that cell.
- Reveal all non-mine cells to win.
- Flag cells you suspect contain mines.

### Complex
- There are four types of mines: +Real, -Real, +Imag, -Imag.
- Each number shows |sum of adjacent mines|^2 — the squared magnitude of the complex sum.
- Positive and negative mines of the same type can cancel each other out.
- A "0" means all adjacent mines form canceling pairs; a blank cell means no adjacent mines at all.
- Right-click cycles through the four flag types.
- Chord when the flag count matches and the real/imaginary ratio is correct.

## Project Structure

```
src/
  main.rs        — Entry point and game loop
  app.rs         — Application state, rendering, input, menus, dialogs
  game.rs        — Core game logic (platform-independent)
  assets.rs      — PNG atlas loading and sprite management
  game_tests.rs  — Unit tests for game logic
assets/
  atlas.png      — Sprite atlas (256x87)
  atlas.json     — Sprite slot definitions
```

## Technology

- **[macroquad](https://github.com/not-fl3/macroquad)** — Cross-platform windowing and 2D rendering
- **[image](https://github.com/image-rs/image)** — PNG decoding
- **[serde](https://serde.rs/) / [serde_json](https://github.com/serde-rs/json)** — JSON parsing
- **[directories](https://github.com/dirs-dev/directories-rs)** — Cross-platform config paths

## License

Code licensed under GPL-3.0. Original Minesweeper image assets are property of Microsoft and are not covered by the GPL-3.0 license. New image assets by Qingyue Xiao, licensed under GPL-3.0.

This program is an independent remake and is not affiliated with Microsoft.
