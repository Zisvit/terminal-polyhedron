# terminal-polyhedron

Terminal 3D wireframe polyhedron viewer written in Rust. Renders rotating polyhedra (cube, icosahedron, dodecahedron) in an alternate-screen TUI using Braille sub-pixel drawing (2×4 dots per terminal cell), perspective projection, and a simple z-buffer.

## Features

- Wireframe rendering with Braille characters for higher effective resolution
- Perspective projection and rotation
- Auto-fit to the current terminal size with centered placement
- Auto-rotation plus manual controls
- Mouse/touchpad controls (via terminal mouse events): drag to rotate, scroll to zoom
- Runtime resize/terminal zoom handling (frame is recomputed from current `cols × rows`)
- `--dump` mode to render a single frame to stdout without TTY/raw mode (useful for debugging)

## Requirements

- **Rust toolchain** (stable, edition 2021) — install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- A terminal with UTF-8 and Braille/Unicode font support (most modern terminals)

Rust crate dependencies (`crossterm`, `clap`) are declared in `Cargo.toml`/`Cargo.lock` and are **fetched automatically by Cargo** during build — no manual dependency download is needed.

## Install / Download & Run

```bash
# 1) Clone
git clone https://github.com/Zisvit/terminal-polyhedron.git
cd terminal-polyhedron

# 2) Build
cargo build --release

# 3) Run
./target/release/terminal-polyhedron
# or install into your Cargo bin:
# cargo install --path .
# terminal-polyhedron
```

## Build

```bash
cargo build --release
```

Binary: `target/release/terminal-polyhedron`

## Usage

```bash
terminal-polyhedron [OPTIONS]
```

### Options

| Option | Default | Description |
|---|---:|---|
| `--shape <SHAPE>` | `dodecahedron` | `cube`, `icosahedron`, or `dodecahedron` |
| `--fps <FPS>` | `120` | Target frames per second |
| `--scale <SCALE>` | `1.0` | Visual scale multiplier |
| `--distance <DIST>` | `5.0` | Camera distance along Z |
| `--focal <FOCAL>` | `2.0` | Perspective focal length |
| `--subdivisions <N>` | `2` (CLI) / `12` (runtime floor) | Edge subdivisions (higher = smoother perspective on edges) |
| `--speed <RAD_S>` | `0.2` (CLI) / `0.8` (runtime default) | Auto-rotation speed (radians/sec) |
| `--dump` | off | Render one frame to stdout (no TTY) |
| `--cols <N>` | `$COLUMNS`/80 | Columns for `--dump` |
| `--rows <N>` | `$LINES`/24 | Rows for `--dump` |

> Note: runtime rendering enforces a minimum subdivision count for smoother edges. You can still experiment with CLI values; rendering clamps/floors as implemented.

### Examples

```bash
# Run interactive viewer
terminal-polyhedron

# Cube with slower speed
terminal-polyhedron --shape cube --speed 0.4

# Debug a single frame without TTY
terminal-polyhedron --dump --cols 80 --rows 24 --shape dodecahedron
```

## Controls

| Input | Action |
|---|---|
| `q` / `Esc` | Quit |
| `r` | Reset rotation |
| Arrow keys | Rotate X/Y |
| `a` / `d` | Rotate Z |
| `+` / `=` / `-` | Scale up/down |
| Left mouse drag | Rotate X/Y |
| Right mouse drag | Rotate Z |
| Mouse/touchpad scroll | Scale up/down |

Mouse/touchpad behavior depends on terminal emulator support for mouse events in alternate-screen TUI mode.

## Project Structure

```text
.
├── Cargo.toml
├── Cargo.lock
├── README.md
└── src/
    ├── app.rs        # App loop, input handling, frame pacing
    ├── geometry.rs   # Polyhedron definitions (vertices/edges)
    ├── lib.rs        # Library module exports
    ├── main.rs       # CLI entrypoint, --dump mode
    ├── math.rs       # Vec3, rotation, projection helpers
    ├── render.rs     # Braille rasterization, projection-to-screen, fit/centering
    └── terminal.rs   # Terminal setup/restore, mouse capture, event polling
```

## Notes

- Rendering is designed to keep the model centered within the current terminal bounds and adapt to size changes (including font zoom that changes terminal `cols/rows`).
- Visual results can vary slightly by terminal emulator/font/`$TERM` behavior, especially for mouse event reporting.
- `target/` contains build artifacts and is not part of source.

## License

No license file is currently included. Add one if you plan to distribute the project.
