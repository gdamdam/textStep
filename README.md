<div align="center">

# T E X T S T E P

**A terminal-based step sequencer, drum machine, and synthesizer built entirely in Rust.**

All DSP from scratch — no samples, no external audio libraries. Just your terminal and your speakers.

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Platform: macOS](https://img.shields.io/badge/Platform-macOS-lightgrey?logo=apple&logoColor=white)](https://www.apple.com/macos/)
[![Audio: CoreAudio](https://img.shields.io/badge/Audio-CoreAudio-green)](https://developer.apple.com/documentation/coreaudio)
[![TUI: ratatui](https://img.shields.io/badge/TUI-ratatui-purple)](https://ratatui.rs/)
[![Lines of Code](https://img.shields.io/badge/Lines_of_Code-17k-informational)]()
[![Tests](https://img.shields.io/badge/Tests-23_passing-brightgreen)]()

![TextStep Demo](assets/demo.gif)

</div>

---

## Features

- **8 Drum Tracks** — Kick, Snare, Closed HiHat, Open HiHat, Ride, Clap, Cowbell, Tom — each fully synthesized with 8 tweakable sound parameters
- **Polyphonic Synth** — 2 oscillators + sub, 2 ADSR envelopes, resonant filter, LFO with 6 waveforms, collapsible UI section
- **32-Step Sequencer** — 10 patterns and 8 kit slots with per-pattern BPM and swing
- **Send Effects Chain** — Schroeder reverb, tempo-synced filtered delay, tube saturator, SSL-style glue compressor
- **Live Performance** — drum pads, real-time recording, pattern queuing, per-pattern BPM
- **Mouse Support** — click the grid, drag parameters Ableton-style, audition sounds from the activity bar
- **Project System** — save/load `.tsp` files, standalone kit export, preset browser
- **Spectrum Analyzer** — real-time FFT spectrum display and VU meter with 90s Hi-Fi LED aesthetic

Ships with **10 demo patterns** ready to play: House, Chicago House, Brit House, French House, Dirty House, Trance, Techno, Drum & Bass, Trap, and Moombahton.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.70 or later
- macOS with CoreAudio (primary target)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/illobo/textStep.git
cd textStep

# Build and run (release mode recommended for audio performance)
cargo build --release
cargo run --release
```

### Run Tests

```bash
cargo test    # 23 tests, runs in <1s
```

## Quick Manual

### Transport

| Key | Action |
|-----|--------|
| `Space` | Play / Pause |
| `Esc` | Stop (reset to step 0) |
| `-` / `=` | BPM -1 / +1 |
| `_` / `+` | BPM -10 / +10 |
| `` ` `` | Toggle record mode |
| `l` | Toggle loop on/off |
| `L` | Cycle loop length: 8 / 16 / 24 / 32 |
| `Shift+C` | Cycle compressor: Off / Light / Medium / Heavy / Max |

### Navigation

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Cycle focus: Grid → Controls → Transport |
| Arrow keys | Move cursor in grid or controls |
| `Enter` | Toggle step (and advance — hold to fill) |
| `;` | Cycle parameter page: SYN → AMP → FX |
| `F2` | Collapse/expand synth section |
| `~` | Toggle spectrum analyzer / VU meter |
| `?` | Help overlay |

### Sound Design

Each drum track has 8 parameters across three pages:

| Page | Parameters | Controls |
|------|------------|----------|
| **SYN** | Tune, Sweep, Color, Snap | Pitch, timbre, transient character |
| **AMP** | Filter, Drive, Decay, Volume | Tone shaping, saturation, envelope |
| **FX** | Reverb Send, Delay Send | Per-track effect routing |

Tweak with `Shift+Up/Down` (adjust value) or `Alt+Up/Down` (adjust and audition simultaneously). With the mouse, click-drag any gauge vertically. `Alt+R` randomizes the current page across all tracks.

Mute (`Shift+M`) and Solo (`Shift+S`) are always accessible on any page.

### Drum Pads

The bottom keyboard row triggers sounds live:

| `z` | `x` | `c` | `v` | `b` | `n` | `m` | `,` |
|-----|-----|-----|-----|-----|-----|-----|-----|
| Kick | Snare | CHH | OHH | Ride | Clap | Cowbell | Tom |

With record enabled and playback running, pad hits write steps at the playhead.

### Patterns & Kits

**Patterns** — 10 slots, each with its own step data and BPM:

| Key | Action |
|-----|--------|
| `q` `w` `e` `r` `t` `y` `u` `i` `o` `p` | Queue pattern 1–10 (switches at loop end) |
| `Shift+` above | Switch pattern immediately |
| `[` / `]` | Queue prev / next |
| `{` / `}` | Immediate prev / next |

**Kits** — 8 slots of sound parameters, shared across patterns:

| Key | Action |
|-----|--------|
| `1` through `8` | Switch to kit slot |

### Synth

The synth section (toggle visibility with `F2`) provides a polyphonic synthesizer with 2 main oscillators, a sub-oscillator, noise generator, two ADSR envelopes, a 24dB resonant filter, and an LFO with 6 waveforms. Synth notes are triggered with `z` `x` `c` `v` when the synth grid is focused, with `Up/Down` for pitch and `(` `)` for octave shifts.

### File Operations

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save project |
| `Ctrl+O` | Load project |
| `Ctrl+N` | Rename current pattern |
| `Ctrl+K` | Save kit as standalone `.tsk` file |
| `Ctrl+J` | Load kit into active slot |
| `Ctrl+P` | Preset browser |
| `Ctrl+L` | Pattern browser |
| `Ctrl+C` / `Ctrl+Q` | Quit |

Projects are stored as JSON in `~/Library/Application Support/textstep/projects/`.

## Architecture

```
┌──────────────────────────────────────────────────┐
│                   UI Thread                       │
│  ┌────────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  ratatui   │  │ crossterm│  │   App State   │  │
│  │  renderer  │  │  events  │  │  (app.rs)     │  │
│  └────────────┘  └──────────┘  └──────────────┘  │
│                                                    │
│  crossbeam channels (lock-free, bounded)           │
│         ▼ UiToAudio          ▲ AudioToUi           │
│                                                    │
│                 Audio Thread                       │
│  ┌────────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  Sequencer │  │  Voices  │  │   Effects    │  │
│  │   Clock    │  │ Drum+Syn │  │ Rev/Dly/Comp │  │
│  └────────────┘  └──────────┘  └──────────────┘  │
│                       │                            │
│                  cpal/CoreAudio                    │
└──────────────────────────────────────────────────┘
```

**Two-thread model:**

- **UI thread** — ratatui + crossterm for rendering and input at ~60fps
- **Audio thread** — cpal/CoreAudio callback running all DSP per-sample

Communication is lock-free via bounded crossbeam channels. The audio thread never blocks.

### DSP — All From Scratch

Every sound is synthesized in real-time with no external DSP dependencies:

- **Drum voices** — TR-808/909-inspired kicks (sine + pitch envelope + resonant impulse), noise-blended snares, 6-oscillator metallic banks for hats and rides (Mutable Instruments Plaits-style inharmonic ratios), ring-modulated open hats, bandpass claps, detuned pulse cowbells, FM toms
- **Synth voice** — dual oscillators, sub, noise, two ADSR envelopes, resonant SVF filter, 6-waveform LFO
- **Effects** — Schroeder/Freeverb reverb (4 comb + 2 allpass), tempo-synced filtered delay, asymmetric tube saturator, feedforward RMS glue compressor with soft knee
- **Primitives** — 1-pole HP/LP filters, state-variable filter, xorshift32 noise, tanh waveshaping

### Source Map

| Directory | Purpose |
|-----------|---------|
| `src/` | Core: entry point, app state, input handling, messages |
| `src/ui/` | Rendering: layout, theme, grids, knobs, transport, spectrum |
| `src/audio/` | DSP: engine, clock, drum/synth voices, effects, mixer, FFT |
| `src/sequencer/` | Data: patterns, transport, project serialization |
| `src/presets/` | Preset browser: drum/synth sounds and patterns by genre |

### Color Palette

Hardware/synthwave aesthetic — all rendered with UTF-8 block characters on a dark background.

| Color | Hex | Usage |
|-------|-----|-------|
| Amber | `#e8a838` | Active steps, gauge fills, primary data |
| Cyan | `#61dafb` | Transport state, beat LEDs, playhead, focused borders |
| Pink | `#ff6b9d` | Focus/selection, current track, record |
| Gold | `#ffd700` | Queued patterns, warnings |

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.29 | TUI rendering framework |
| `crossterm` | 0.28 | Terminal backend (events, raw mode) |
| `cpal` | 0.15 | Cross-platform audio I/O (CoreAudio) |
| `crossbeam-channel` | 0.5 | Lock-free bounded MPSC channels |
| `serde` + `serde_json` | 1 | Project serialization (JSON) |

No other runtime dependencies.

## Project Structure

```
textStep/
├── src/
│   ├── main.rs              # Entry point, thread spawning, event loop
│   ├── app.rs               # Application state, focus, modals
│   ├── keys.rs              # Keyboard input handler
│   ├── mouse.rs             # Mouse input handler (mirrors UI layout)
│   ├── messages.rs          # Cross-thread message enums
│   ├── params.rs            # Shared effect parameters
│   ├── ui/
│   │   ├── mod.rs           # Top-level render dispatch
│   │   ├── layout.rs        # Layout constants (single source of truth)
│   │   ├── theme.rs         # Color palette and styling
│   │   ├── transport_bar.rs # Transport controls rendering
│   │   ├── drum_grid.rs     # 8×32 drum step grid
│   │   ├── synth_grid.rs    # Synth step grid with note display
│   │   ├── knobs.rs         # Drum parameter sliders
│   │   ├── synth_knobs.rs   # Synth parameter groups
│   │   ├── waveform.rs      # Spectrum analyzer / VU meter
│   │   ├── splash.rs        # Boot animation
│   │   └── help_overlay.rs  # Keyboard shortcut reference
│   ├── audio/
│   │   ├── engine.rs        # Audio callback and voice management
│   │   ├── clock.rs         # Beat/step timing with swing
│   │   ├── drum_voice.rs    # 8 drum synth voices (all DSP)
│   │   ├── synth_voice.rs   # Polyphonic synth DSP
│   │   ├── effects.rs       # Reverb, delay, compressor, saturator
│   │   ├── mixer.rs         # Channel mixing, mute/solo
│   │   ├── display_buffer.rs # Lock-free audio→UI waveform buffer
│   │   └── fft.rs           # FFT for spectrum analyzer
│   ├── sequencer/
│   │   ├── drum_pattern.rs  # Drum pattern data (8 tracks × 32 steps)
│   │   ├── synth_pattern.rs # Synth pattern data and parameters
│   │   ├── transport.rs     # Transport state (BPM, play, swing)
│   │   └── project.rs       # Project serialization (.tsp JSON)
│   └── presets/
│       ├── mod.rs           # Preset browser state machine
│       ├── drum_presets.rs   # Drum sound presets
│       ├── synth_presets.rs  # Synth sound presets
│       ├── pattern_presets.rs      # Drum pattern presets by genre
│       └── synth_pattern_presets.rs # Synth pattern presets
├── assets/
│   └── demo.gif             # Demo recording
├── Cargo.toml
├── BLUEPRINT.md             # Full technical documentation
└── LICENSE                  # GPL v2
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test`
2. Code compiles without warnings: `cargo build --release`
3. Follow the existing code style and comment conventions

See [BLUEPRINT.md](BLUEPRINT.md) for full technical documentation and architecture details.

## License

This project is licensed under the [GNU General Public License v2.0](LICENSE).

---

<div align="center">

**Built with Rust** · **All DSP from scratch** · **Zero audio dependencies**

</div>
