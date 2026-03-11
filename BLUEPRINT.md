# BLUEPRINT.md — TextStep: TUI Drum Machine

> Updated: March 2026 — reflects the current implemented state of the project.

## 1. Project Overview

**TextStep** is a terminal-based (TUI) drum machine built in Rust. It runs entirely in the terminal using `ratatui` for rendering and `cpal` for real-time audio output on macOS (CoreAudio).

All drum sounds are **synthesized from scratch** — no samples, no external DSP crates. The architecture follows a strict **two-thread model**: a UI thread for rendering and input, and a real-time audio thread running DSP inside the `cpal` callback. Communication is lock-free via `crossbeam` bounded channels.

### Key Features

- 32-step drum machine with 8 synthesized tracks (Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom)
- **8 sound parameters per track** (Syntakt-inspired): Tune, Sweep, Color, Snap, Filter, Drive, Decay, Volume — each interpreted per-voice for maximum tonal range
- **Send effects**: per-track reverb and delay sends (Schroeder/Freeverb reverb + tempo-synced filtered delay)
- **Master glue compressor**: SSL G-bus inspired feedforward RMS compressor with soft knee, single "amount" knob, displayed in transport bar
- Three-page parameter view (SYN / AMP / FX) with Mute/Solo always visible
- Drum pad keys (ZXCVBNM,) for live triggering and real-time recording at the playhead
- **Mouse support**: click to move cursor, double-click to toggle steps, click+drag parameters (Ableton-style), click activity bar pads to audition, click pattern/kit selectors, drag compressor knob
- **Project system**: 8 kits + 10 patterns saved as `.tsp` JSON files, with hex-encoded step storage
- **Per-pattern BPM**: each pattern stores its own tempo, applied automatically on switch
- **Demo project**: 10 pre-loaded genre patterns (House, Chicago House, Brit House, French House, Dirty House, Trance, Techno, D&B, Trap, Moombahton) with appropriate BPMs
- **Kit bank**: 8 kits switchable via keys 1-8, with magenta highlight in transport bar
- Pattern selection via QWERTYUIOP with queued (end-of-loop) or immediate switching
- Transport: Play/Pause/Stop, BPM control (60-300), beat LED, loop with configurable step length
- **Splash screen**: ASCII logo slide-in animation with matrix rain reveal effect; any key or mouse click to skip
- Save/Load dialogs (Ctrl+S / Ctrl+O), file picker, dirty flag in title bar
- Activity bar with per-track trigger flash indicators and live parameter display
- Help overlay with 2-column key binding reference (~22 rows, shorthand S-/A-/C- notation)

---

## 2. Module Structure

```
textstep/
├── Cargo.toml
├── BLUEPRINT.md
├── src/
│   ├── main.rs                 # Entry point: init terminal + mouse capture, spawn audio, splash loop, event loop
│   ├── app.rs                  # App struct, UiState, MouseState, SplashState, FocusSection, ParamPage, field enums, modals
│   ├── keys.rs                 # Keyboard input mapping, modal handlers, navigation logic
│   ├── mouse.rs                # Mouse input handling: hit testing, click, double-click, drag
│   ├── messages.rs             # UiToAudio and AudioToUi message enums
│   ├── params.rs               # Shared parameter types (MasterParams, EffectParams)
│   │
│   ├── audio/
│   │   ├── mod.rs              # cpal stream setup, start_audio_stream()
│   │   ├── engine.rs           # AudioEngine: callback closure, voice mixing, send bus, compressor, clock
│   │   ├── clock.rs            # SequencerClock: sample-accurate step advancement
│   │   ├── drum_voice.rs       # DrumVoiceDsp trait + 8 voice implementations + apply_drive()
│   │   ├── effects.rs          # ReverbEffect + DelayEffect + GlueCompressor
│   │   └── mixer.rs            # Mute/solo logic, soft_clip(tanh)
│   │
│   ├── sequencer/
│   │   ├── mod.rs              # Re-exports
│   │   ├── drum_pattern.rs     # DrumPattern: 8 tracks x 32 steps + DrumTrackParams (8 params + 2 sends)
│   │   ├── transport.rs        # PlayState, RecordMode, LoopConfig, Transport
│   │   └── project.rs          # ProjectFile, DrumKit, PatternData, hex encoding, file I/O, demo_project()
│   │
│   └── ui/
│       ├── mod.rs              # Top-level render dispatch, activity bar, modal dialogs
│       ├── drum_grid.rs        # Drum grid: 8x32 steps + page-aware inline track controls
│       ├── splash.rs           # Splash screen: logo slide-in + matrix rain reveal
│       ├── transport_bar.rs    # Play state, BPM, compressor gauge, loop, record, beat LED, pattern selector
│       ├── help_overlay.rs     # Centered popup with key binding reference
│       └── theme.rs            # Color palette, style constants, gauge helpers
```

### Dependencies (Cargo.toml)

```toml
[package]
name = "textstep"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.29"          # TUI rendering framework
crossterm = "0.28"        # Terminal backend (raw mode, events, mouse capture)
cpal = "0.15"             # Cross-platform audio (CoreAudio on macOS)
crossbeam-channel = "0.5" # Lock-free bounded channels for UI<->Audio
serde = { version = "1", features = ["derive"] }   # Serialization for project files
serde_json = "1"          # JSON format for .tsp project files
```

---

## 3. Thread Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      MAIN THREAD                             │
│  crossterm events → keys::handle_key() → App state           │
│                   → mouse::handle_mouse() → App state        │
│  App::tick() drains rx_from_audio, decays flashes/status     │
│  ui::render() → ratatui Frame → terminal                     │
│                    ↓ tx_to_audio    ↑ rx_from_audio          │
└────────────────────┼────────────────┼────────────────────────┘
                     │  crossbeam     │
                     │  bounded(64)   │  bounded(16)
┌────────────────────┼────────────────┼────────────────────────┐
│                    ↓                ↑                         │
│  AudioEngine::process(buffer)                                │
│    1. Drain UI messages (try_recv)                           │
│    2. Per sample: advance clock → trigger voices → mix       │
│    3. Send PlaybackPosition to UI (try_send, drop if full)   │
│                  AUDIO THREAD (cpal callback)                │
└──────────────────────────────────────────────────────────────┘
```

- **UI->Audio channel**: bounded(64) — carries `UiToAudio` messages (transport, drum pattern, effect params, trigger)
- **Audio->UI channel**: bounded(16) — carries `AudioToUi::PlaybackPosition` with trigger bitmask
- Messages are non-blocking: `try_recv` in audio callback, `try_send` drops if full
- **Initial state**: `App::new()` sends `SetTransport` and `SetDrumPattern` to the audio thread during construction so the engine has the demo pattern before the first play
- Event loop drains all pending crossterm events per frame for responsive key repeat
- Mouse events (`Event::Mouse`) are handled alongside keyboard events in the same drain loop

---

## 4. Core Data Structures

### 4.1 Transport (`src/sequencer/transport.rs`)

```rust
pub enum PlayState { Stopped, Playing, Paused }
pub enum RecordMode { Off, On }

pub struct LoopConfig {
    pub enabled: bool,          // default: true
    pub length_steps: u8,       // 8, 16, 24, or 32 (default: 32)
}

pub struct Transport {
    pub state: PlayState,       // default: Stopped
    pub bpm: f64,               // 60.0..=300.0, default: 120.0
    pub record_mode: RecordMode,
    pub loop_config: LoopConfig,
}
```

### 4.2 Drum Pattern (`src/sequencer/drum_pattern.rs`)

```rust
pub const NUM_DRUM_TRACKS: usize = 8;
pub const MAX_STEPS: usize = 32;

pub enum DrumTrackId { Kick, Snare, ClosedHiHat, OpenHiHat, Ride, Clap, Cowbell, Tom }

pub struct DrumTrackParams {
    // Synthesis (SYN page)
    pub tune: f32,    // 0.0..=1.0  Pitch / frequency center
    pub sweep: f32,   // 0.0..=1.0  Pitch envelope depth / ring mod depth / detune
    pub color: f32,   // 0.0..=1.0  Timbre: pitch env time, phase mod, noise color
    pub snap: f32,    // 0.0..=1.0  Transient click/attack character

    // Filter / Shape (AMP page)
    pub filter: f32,  // 0.0..=1.0  Filter cutoff frequency (LP/HP per voice)
    pub drive: f32,   // 0.0..=1.0  Saturation / overdrive (waveshaping)

    // Amplitude (AMP page)
    pub decay: f32,   // 0.0..=1.0  Amplitude envelope decay time
    pub volume: f32,  // 0.0..=1.0  Track output level

    // Send effects (FX page)
    pub send_reverb: f32, // 0.0..=1.0  Send level to reverb
    pub send_delay: f32,  // 0.0..=1.0  Send level to delay

    // Runtime state (not saved in kits)
    pub mute: bool,
    pub solo: bool,
}

pub struct DrumPattern {
    pub steps: [[bool; MAX_STEPS]; NUM_DRUM_TRACKS],
    pub params: [DrumTrackParams; NUM_DRUM_TRACKS],
}
```

Each `DrumTrackId` has its own tuned defaults via `DrumTrackParams::defaults_for()`, so a fresh kit sounds good immediately.

### 4.3 Project System (`src/sequencer/project.rs`)

```rust
pub const NUM_PATTERNS: usize = 10;
pub const NUM_KITS: usize = 8;

/// Serializable sound params (8 synth + 2 send, no mute/solo). Forward-compatible via #[serde(default)].
pub struct DrumSoundParams {
    pub tune: f32, pub sweep: f32, pub color: f32, pub snap: f32,
    pub filter: f32, pub drive: f32, pub decay: f32, pub volume: f32,
    pub send_reverb: f32, pub send_delay: f32,  // #[serde(default)] for old files
}

pub struct DrumKit {
    pub name: String,
    pub tracks: Vec<KitTrack>,  // KitTrack { id: String, params: DrumSoundParams }
}

/// Hex-encoded steps per track (8 hex chars = 32 steps, 4 steps per nibble).
pub struct PatternData {
    pub name: String,
    pub bpm: f64,               // Per-pattern BPM override (0.0 = use project BPM)
    pub steps: Vec<String>,     // e.g. ["88880000", "00008888", ...]
}

pub struct ProjectFile {
    pub textstep: FileHeader,   // { format_version: 1, app_version }
    pub metadata: ProjectMetadata, // { name, author }
    pub kit: DrumKit,           // Legacy single kit (read-only, skip_serializing for backward compat)
    pub kits: Vec<DrumKit>,     // Kit bank: 8 kits
    pub active_kit: usize,      // 0-7
    pub patterns: Vec<PatternData>, // 10 patterns
    pub active_pattern: usize,
    pub bpm: f64,               // Project default BPM
    pub loop_length: u8,
    pub effects: EffectParams,  // Global reverb/delay/compressor params (#[serde(default)])
}
```

- **File format**: `.tsp` (JSON, pretty-printed)
- **Data directory**: `~/Library/Application Support/textstep/projects/` (macOS), `$XDG_DATA_HOME/textstep/` (Linux)
- **Forward compatibility**: all fields use `#[serde(default)]` so old files with fewer params load cleanly
- `normalize()` ensures exactly 10 patterns exist after loading

### 4.4 Message Types (`src/messages.rs`)

```rust
pub enum UiToAudio {
    SetTransport(Transport),
    SetDrumPattern(DrumPattern),
    SetEffectParams(EffectParams), // global reverb/delay/compressor params
    TriggerDrum(usize),            // track index — fire voice immediately (drum pads)
}

pub enum AudioToUi {
    PlaybackPosition {
        drum_step: usize,
        beat: u8,            // 0..3
        is_bar_start: bool,
        triggered: u8,       // bitmask: which tracks triggered on this step
    },
}
```

### 4.5 UI State & App (`src/app.rs`)

```rust
pub enum FocusSection { DrumGrid, DrumTrackControls, Transport }

pub enum ParamPage {
    Synth,  // Tune, Sweep, Color, Snap + Mute/Solo
    Amp,    // Filter, Drive, Decay, Volume + Mute/Solo
    Fx,     // SendReverb, SendDelay + Mute/Solo
}

pub enum DrumControlField {
    Tune, Sweep, Color, Snap,       // SYN page
    Filter, Drive, Decay, Volume,   // AMP page
    SendReverb, SendDelay,          // FX page
    Mute, Solo,                     // always visible
}

pub enum ModalState {
    None,
    TextInput { prompt, buffer, on_confirm: ModalAction },
    FilePicker { title, items: Vec<(String, PathBuf)>, selected, on_confirm: ModalAction },
}

pub enum ModalAction { SaveProject, RenamePattern, SaveKit, LoadProject, LoadKit }

/// Splash screen animation phases: SlideIn → Hold → MatrixReveal → Done
pub enum SplashPhase { SlideIn, Hold, MatrixReveal, Done }

pub struct SplashState {
    pub phase: SplashPhase,
    pub frame: u16,
    pub matrix_columns: Vec<(f32, f32, u8)>,  // per-column: (row, speed, char)
    pub revealed: Vec<bool>,                   // row-major reveal mask
    pub matrix_width: u16,
    pub matrix_height: u16,
}

/// State for parameter drag interaction (Ableton-style vertical drag).
pub struct DragState {
    pub track: usize,
    pub field: DrumControlField,
    pub start_y: u16,
    pub start_value: f32,
}

/// State for compressor knob drag.
pub struct CompressorDrag {
    pub start_y: u16,
    pub start_value: f32,
}

/// Mouse interaction state.
pub struct MouseState {
    pub last_click: Option<(Instant, usize, usize)>,  // double-click detection (time, track, step)
    pub drag: Option<DragState>,                        // parameter drag
    pub compressor_drag: Option<CompressorDrag>,        // compressor knob drag
}

pub struct UiState {
    pub splash: SplashState,
    pub focus: FocusSection,
    pub drum_cursor_track: usize,
    pub drum_cursor_step: usize,
    pub drum_ctrl_track: usize,
    pub drum_ctrl_field: DrumControlField,
    pub param_page: ParamPage,
    pub playback_step: usize,
    pub current_beat: u8,
    pub is_bar_start: bool,
    pub show_help: bool,
    pub trigger_flash: [u8; 8],     // per-track flash countdown (~100ms at 60fps)
    pub active_pattern: usize,      // 0-9
    pub queued_pattern: Option<usize>,
    pub active_kit: usize,          // 0-7
    pub modal: ModalState,
    pub status_msg: Option<StatusMessage>,
    pub mouse: MouseState,          // mouse interaction state
}

pub struct App {
    pub ui: UiState,
    pub transport: Transport,
    pub drum_pattern: DrumPattern,
    pub master: MasterParams,
    pub effect_params: EffectParams,
    pub project: ProjectFile,
    pub project_path: Option<PathBuf>,
    pub dirty: bool,                // shown as * in title bar
    pub tx_to_audio: Sender<UiToAudio>,
    pub rx_from_audio: Receiver<AudioToUi>,
    pub should_quit: bool,
}
```

---

## 5. Audio Engine

### 5.1 AudioEngine (`src/audio/engine.rs`)

Owns all DSP state. Called from the cpal callback via `process(&mut self, buffer: &mut [f32])`.

**Per-buffer flow:**
1. Drain all pending `UiToAudio` messages (non-blocking)
2. Handle `TriggerDrum(track)` — trigger voice immediately with current params, apply hihat choke
3. Handle `SetEffectParams` — update reverb/delay parameters and compressor amount
4. Handle `SetTransport` — detect BPM changes and update delay time accordingly
5. Build mute/solo arrays for `effective_mute()`

**Per-sample flow (stereo interleaved):**
1. If Playing: advance `SequencerClock`
2. On step event:
   - Trigger active drum voices with `voice.trigger(&params)` (respecting mute/solo)
   - Hihat choke: closed hihat silences open hihat
   - Send `PlaybackPosition` to UI with `triggered` bitmask
3. Sum drum voices (each × track volume) into dry bus
4. Tap each voice post-volume into reverb/delay send buses (per-track send levels)
5. Process send effects (reverb, delay) and sum wet returns
6. Mix dry + wet → master volume → **glue compressor** → `soft_clip(tanh)` → stereo output

### 5.2 Sequencer Clock (`src/audio/clock.rs`)

Sample-accurate step timing: `samples_per_step = sample_rate * 60.0 / bpm / 4.0`

- First call after `new()` or `reset()` immediately returns step 0
- Steps wrap at `loop_length` (configurable: 8/16/24/32)
- `StepEvent` includes: `drum_step`, `beat` (0..3), `is_bar_start`

### 5.3 Drum Voices (`src/audio/drum_voice.rs`)

All voices implement `DrumVoiceDsp` trait: `trigger(&DrumTrackParams)`, `choke()`, `tick() -> f32`. Sample rate is set at construction time. Every voice uses `apply_drive()` for waveshaping saturation.

#### Universal Parameter Interpretation (per-voice)

Each of the 8 params is interpreted differently by each voice, inspired by the Elektron Syntakt approach:

| Param | Kick | Snare | CHH | OHH | Ride | Clap | Cowbell | Tom |
|---|---|---|---|---|---|---|---|---|
| **tune** | Fundamental freq (30-80Hz) | Body freq (120-280Hz) | Metallic bank base (300-900Hz) | Metallic resonance base (200-800Hz) | Osc bank base (150-600Hz) | BPF center (800-4kHz) | Base freq (~545-645Hz) | Fundamental (60-350Hz) |
| **sweep** | Pitch env depth (0-300Hz) | Body pitch env depth | Noise layer mix (0.3-1.0) | Ring mod depth (noise↔metallic) | Inharmonicity spread | Punch transient boost | Oscillator detune ratio | Pitch env depth |
| **color** | Pitch env time (5-80ms) | Tone/noise balance | Cross-FM intensity (0-1.5) | Phase mod depth (clean↔gritty) | FM cross-modulation (0-2.0) | Noise color (white→pink) | Pulse width (0.3-0.7) | FM depth/grit |
| **snap** | Click impulse level | Impact transient amp | Transient click burst | Stick transient | Ping bell transient | Burst count (3-6) | Attack pop noise | Waveshape + click |
| **filter** | LP on body (500-8kHz) | Tightness gate | HP cutoff (3-8kHz) + LP (6-18kHz) | HP (4-10kHz) + sweeping LP | HP cutoff (airiness) | BPF resonance | BP width | LP on output (200-8kHz) |
| **drive** | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation | Waveshaping saturation |
| **decay** | Body amp env (80-600ms) | Noise tail (50-400ms) | CHH: 10-80ms | OHH: 100-800ms (two-band) | 200ms-2s (two-band) | Tail (50-300ms) | 50-300ms | 80-500ms |
| **volume** | Track level | Track level | Track level | Track level | Track level | Track level | Track level | Track level |

#### Voice Synthesis Techniques

| Voice | Core Synthesis |
|---|---|
| **Kick** | TR-909 inspired two-path design. **Path A**: Sine oscillator with cubed pitch envelope (`tune` sets fundamental, `sweep` sets pitch drop depth, `color` sets pitch env decay time). Body LP filter + amplitude envelope. **Path B**: Low-frequency square impulse (DC pulse) → resonant LP filter (~5kHz, 18% resonance) for musical "knock" character. Fixed short click envelope (~4ms). `snap` controls click level ("Attack" knob). Both paths summed. |
| **Snare** | Sine body (separate short decay) + HP-filtered noise (longer decay via `decay`). Impact transient (~3ms raw noise burst). Tightness gate via `filter` (pow envelope shaping). |
| **Closed HiHat** | 6-oscillator metallic bank (TR-808/Plaits-style square waves at `CYMBAL_RATIOS`). Cross-FM between oscillators (`color`). Alternating add/multiply mixing. Noise layer for fizz (`sweep`). HP→SVF LP filter chain (3-8kHz HP, 6-18kHz LP). Short decay 10-80ms. |
| **Open HiHat** | **Ring modulation** design: white noise × 6-oscillator metallic bank (sine-based with phase modulation). The metallic bank acts as a spectral template on noise, producing "shhhhh with shimmer" rather than pure metallic tones. `sweep` controls ring mod depth (pure noise ↔ metallic coloring). Random phase initialization per trigger for natural variation. Two-band envelope (HF decays 3× faster). HP (4-10kHz) + sweeping LP filter chain (starts bright, darkens over time). Smooth exponential choke (~10ms fade) by CHH. ~0.5ms attack ramp prevents onset click. |
| **Ride** | 6-oscillator metallic bank with wider inharmonicity spread. Cross-FM (`color`). Ping transient (sine burst at 2.5× base, ~8ms). Two-band envelope (HF decays 3× faster than body). HP→SVF LP filter chain. Longer decay range 200ms-2s. |
| **Clap** | Resonant SVF bandpass filter (true resonance via `filter`). Noise color blend white↔pink (`color`). 3-6 rapid on/off bursts (`snap`). Punch transient boost/compress (`sweep`). |
| **Cowbell** | Two detuned pulse waves (variable width via color) through bandpass. Sweep controls detune ratio. Snap adds pop. |
| **Tom** | FM synthesis (same architecture as Kick). Wider freq range (60-350Hz). Cubed pitch envelope. Cubic waveshaping. Reduced noise mix. LP filter on output. |

#### `apply_drive()` Waveshaping

```rust
fn apply_drive(x: f32, drive: f32) -> f32 {
    let gain = 1.0 + drive * 8.0;
    (x * gain).tanh() / gain.tanh()
}
```

Drive=0 is clean passthrough, drive=1 is heavy saturation with gain of 9x into normalized tanh.

#### DSP Helpers

- `Noise`: xorshift32 PRNG, returns -1.0..1.0, one instance per voice
- `OnePoleHP`: 1-pole highpass filter with `set_freq(hz, sr)`
- `OnePoleLP`: 1-pole lowpass filter with `set_freq(hz, sr)`
- `StateVariableFilter`: 2-pole SVF with LP/HP/BP outputs, resonance control. Ported from zicbox. Used by metallic voices (CHH, OHH, Ride), Clap, and Kick click path.
- `CYMBAL_RATIOS`: inharmonic frequency ratios `[1.0, 1.304, 1.466, 1.787, 1.932, 2.536]` for metallic oscillator bank (Mutable Instruments Plaits). Shared by CHH, OHH, and Ride voices.

### 5.4 Effects (`src/audio/effects.rs`)

#### Send Effects

Two send effects processed in parallel, fed by per-track send levels:

```
8 voices ──┬── dry sum (per-track volume) ──────────────────┐
           │                                                 │
           ├── reverb send (per-track send level) → Reverb ──┤ → mix → master → compressor → soft_clip → out
           │                                                 │
           └── delay send (per-track send level) → Delay ───┘
```

#### Reverb (Schroeder/Freeverb topology)

Ported from zicbox `applyReverb()`:
- **4 parallel comb filters** with damped feedback at prime-number delay lengths (1117, 1301, 1571, 1787 samples)
- **2 series allpass filters** for diffusion (557, 443 samples)
- ~26KB buffer total, ~12 float ops per sample
- Damping: one-pole LP in each comb feedback path for natural high-frequency rolloff

Parameters:
- `amount` (0.0-1.0): controls feedback (0.50-0.92 range) and wet mix
- `damping` (0.0-1.0): high-frequency absorption in comb feedback (0 = bright, 1 = dark)

#### Delay (filtered feedback delay)

Tempo-synced delay line with LP-filtered feedback:
- Circular buffer (~131072 samples ≈ 2.7s at 48kHz, ~512KB)
- Delay time derived from BPM and subdivision (1/8, 1/4, dotted 1/8, 1/3)
- One-pole LP in feedback loop for tape-echo character (highs roll off each repeat)

Parameters:
- `time` (0.0-1.0): subdivision selector (quantized to musical values based on BPM)
- `feedback` (0.0-1.0): feedback amount (clamped to 0.95 max for stability)
- `tone` (0.0-1.0): LP cutoff in feedback loop (1000-12000 Hz)

#### Glue Compressor (SSL G-bus inspired)

Feedforward RMS compressor with soft knee, inserted after master volume and before soft_clip:

```
dry + reverb + delay → master volume → GlueCompressor → soft_clip(tanh) → output
```

**Signal flow**: input → RMS detector (exponential moving average, ~10ms window) → gain computer (soft knee, 6dB knee width) → gain smoother (separate attack/release) → VCA → auto makeup → output

Single "amount" knob (0.0-1.0) maps to all parameters:

| Amount | Threshold | Ratio | Attack | Release | Character |
|--------|-----------|-------|--------|---------|-----------|
| 0.0 | bypass | — | — | — | Clean |
| 0.25 | -7.5 dB | 2.5:1 | 8ms | 175ms | Light glue |
| 0.50 | -13 dB | 3:1 | 6.5ms | 150ms | Medium glue |
| 0.75 | -18.5 dB | 3.5:1 | 4.75ms | 125ms | Heavy pump |
| 1.0 | -24 dB | 4:1 | 3ms | 100ms | Squash |

- **RMS detection**: exponential moving average of squared input (~10ms window), converted to dB
- **Soft knee**: 6dB quadratic interpolation zone around threshold for transparent compression
- **Attack/release**: separate smoothing coefficients on the gain signal. Attack lets transients punch through before compressing; release is musical (no pumping).
- **Auto makeup gain**: compensates for ~50% of theoretical max gain reduction

#### Per-Track Send Routing

Each track gets two additional parameters (3rd parameter page: FX):
- `send_reverb` (0.0-1.0): amount of post-volume signal sent to reverb
- `send_delay` (0.0-1.0): amount of post-volume signal sent to delay

#### Global Effect Parameters

```rust
pub struct EffectParams {
    pub reverb_amount: f32,      // 0.0-1.0
    pub reverb_damping: f32,     // 0.0-1.0
    pub delay_time: f32,         // 0.0-1.0 (mapped to subdivision)
    pub delay_feedback: f32,     // 0.0-1.0
    pub delay_tone: f32,         // 0.0-1.0
    pub compressor_amount: f32,  // 0.0-1.0 (0=off), #[serde(default)]
}
```

Communicated to audio thread via `UiToAudio::SetEffectParams(EffectParams)`. Serialized in project files via `#[serde(default)]` on `ProjectFile.effects`.

### 5.5 Mixer (`src/audio/mixer.rs`)

- `effective_mute()`: if any track is soloed, only soloed tracks play (solo overrides mute)
- `soft_clip()`: `tanh(x)` for output limiting

---

## 6. UI Layout

```
┌──────────────── TextStep - My Project* ──────────────────────┐
│ ▶ PLAY   BPM: 120.0  CMP:████  Loop: [ON] 32 steps  REC: ○ │
│ Beat: ● ○ ○ ○   Pat: [q] w e r t y u i o p  Kit: [1] 2 3 .. │
├─────────────────── DRUM MACHINE [SYN] ──────────────────────┤
│          │1 · · · 2 · · · 3 · · · 4 · · ·│1 · · · 2 · · · │
│ Kick     │■ · · · ■ · · · ■ · · · ■ · · ·│■ · · · ■ · · · │ TN:████ SW:████ CL:████ SN:████ [M] [S]
│ Snare    │· · · · ■ · · · · · · · ■ · · ·│· · · · ■ · · · │ TN:████ SW:████ CL:████ SN:████ [M] [S]
│ CHH      │■ · ■ · ■ · ■ · ■ · ■ · ■ · ■ ·│■ · ■ · ■ · ■ · │ TN:████ SW:████ CL:████ SN:████ [M] [S]
│  ...     │  ...                            │  ...             │  ...
├──────────────────────────────────────────────────────────────┤
│ [Kick] [Snare] [CHH] [OHH] [Ride] [Clap] [Cowbell] [Tom] │ ? Help │  <- activity bar
└──────────────────────────────────────────────────────────────┘
```

### Startup Splash Screen (`src/ui/splash.rs`)

On launch, a 3-phase splash animation plays before the main UI:

1. **SlideIn** (~0.5s, 30 frames): ASCII TextStep logo enters from the left edge with ease-out cubic easing, fading from black to white
2. **Hold** (~1s, 60 frames): Logo sits centered in full white
3. **MatrixReveal** (~2s, max 120 frames): The real drum machine UI renders underneath; matrix rain (green falling characters on black) overlays unrevealed cells and dissolves top-to-bottom as each column's raindrop falls through, revealing the UI behind it

Any keypress or mouse click during any phase skips straight to the main UI. The matrix columns have randomized speeds (0.3-1.5 rows/frame) and staggered start positions for organic feel.

### Layout Constraints (vertical)

| Section | Height | Description |
|---|---|---|
| Transport bar | 4 rows | Play state, BPM, compressor gauge, loop, record, beat LED, pattern selector, kit selector (magenta), dirty flag |
| Drum grid | Min 11 | 1 header + 8 tracks + 2 border. Title shows `[SYN]`/`[AMP]`/`[FX]` page label + pattern name |
| Activity bar | 1 row | Trigger flash pads, parameter display, "? Help" hint, or timed status messages |

### Visual Features

- **Title bar**: "TextStep - {project name}" with `*` dirty indicator
- **Compressor gauge**: `CMP:████` on transport line 1 after BPM — cyan when active, dark gray when off
- **Beat column highlighting**: subtle `Rgb(30,30,40)` background on every 4th step
- **Bar separator**: vertical `│` between steps 15-16 (bar 1 / bar 2)
- **Page-aware inline controls**: SYN page shows `TN:████ SW:████ CL:████ SN:████` / AMP page shows `FL:████ DR:████ DC:████ VL:████` / FX page shows `RV:████ DL:████`, plus `[M] [S]` always visible
- **Focus indication**: cyan border on focused section, dark gray on unfocused
- **Playhead**: yellow background on current step during playback
- **Cursor**: cyan background on selected cell
- **Mute/Solo indicators**: red `M` or green `S` next to track name
- **Pattern selector**: QWERTYUIOP keys shown in transport bar, active=cyan bg, queued=yellow bg
- **Kit selector**: keys 1-8 shown in transport bar next to pattern selector, active=magenta bg
- **Activity bar**: per-track pads flash white-on-black on trigger; shows current parameter tweak when in controls panel; shows "? Help" hint at the end; shows timed status messages (yellow, ~2 seconds)
- **Modal dialogs**: centered popups for text input (save name) and file picker (load), rendered as overlays with yellow borders

---

## 7. Input — Keyboard & Mouse

### Mouse Support (`src/mouse.rs`)

Mouse capture is enabled via crossterm's `EnableMouseCapture` / `DisableMouseCapture`. Hit zones are computed from layout math (deterministic given terminal size + help state) — no stored rects needed.

| Zone | Click | Double-click | Drag |
|------|-------|--------------|------|
| **Drum grid steps** | Move cursor to (track, step) + focus grid | Toggle step on/off (300ms threshold) | — |
| **Inline param gauges** | Select param + focus controls + start drag | — | Adjust value vertically (0.04/row, Ableton-style) |
| **Mute/Solo buttons** | Toggle mute or solo | — | — |
| **Activity bar pads** | Audition sound (TriggerDrum + flash) | — | — |
| **Pattern selector** | Queue pattern switch | — | — |
| **Kit selector** | Switch kit immediately | — | — |
| **Compressor gauge** | Start drag | — | Adjust amount vertically (0.04/row) |
| **Splash screen** | Skip to main UI | — | — |

Double-click detection: tracked via `MouseState.last_click` with `Instant` timestamps. Two clicks within 300ms at the same grid cell = double-click.

Parameter drag: on mouse down over a param gauge, a `DragState` is stored with start Y position and start value. On `MouseEventKind::Drag`, delta Y is computed and the param is adjusted at 0.04 per terminal row (~25 rows for full 0→1 sweep). Mouse up clears the drag state.

### Global Keys (work in any focus section)

| Key | Action |
|---|---|
| `Ctrl+C` / `Ctrl+Q` | Quit |
| `?` | Toggle help overlay |
| `Tab` / `Shift+Tab` | Cycle focus: DrumGrid -> DrumTrackControls -> Transport |
| `Space` | Play / Pause toggle |
| `Esc` | Stop (reset to beginning) |
| `-` / `=` | BPM -1 / +1 |
| `_` / `+` | BPM -10 / +10 |
| `` ` `` (backtick) | Toggle record mode |
| `l` | Toggle loop on/off |
| `L` (Shift) | Cycle loop length: 8 -> 16 -> 24 -> 32 |
| `;` | Cycle param page (SYN → AMP → FX → SYN) |
| `Shift+M` | Toggle mute on current drum track |
| `Shift+S` | Toggle solo on current drum track |
| `Shift+C` | Cycle compressor: Off → Light (0.25) → Medium (0.50) → Heavy (0.75) → Max (1.0) → Off |
| `Ctrl+S` | Save project (prompts for name on first save) |
| `Ctrl+O` | Open load dialog (file picker) |
| `Ctrl+N` | Rename current pattern |
| `Ctrl+K` | Save kit (prompts for name if default) |
| `Ctrl+J` | Load kit (file picker) |
| `Alt+R` | Randomize current page params across all tracks (SYN: sweep/color/snap, AMP: filter/drive/decay, FX: send levels capped at 0.5) |

### Kit Selection

| Key | Action |
|---|---|
| `1` `2` `3` `4` `5` `6` `7` `8` | Switch to kit 1-8 immediately (no status message — magenta highlight is feedback) |

### Pattern Selection

| Key | Action |
|---|---|
| `q w e r t y u i o p` | Queue pattern 1-10 (switches at loop boundary) |
| `Shift+` above | Immediate pattern switch |
| `[` / `]` | Queue prev / next pattern |
| `{` / `}` | Immediate prev / next pattern |

Pattern switching applies per-pattern BPM if set (non-zero `bpm` field in `PatternData`). BPM is saved back to the pattern on every `store_current_to_project()` call.

### Drum Pads (ZXCVBNM,)

| Key | Track |
|---|---|
| `z` `x` `c` `v` `b` `n` `m` `,` | Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom |

- Always triggers the sound immediately via `UiToAudio::TriggerDrum`
- Always flashes the activity bar indicator
- If recording + playing: also writes a step at the current playhead position

### Drum Grid (FocusSection::DrumGrid)

| Key | Action |
|---|---|
| `Left` / `Right` | Move cursor across steps (wraps). Right at step 31 enters track controls. |
| `Up` / `Down` | Move cursor across tracks (wraps, syncs with controls track) |
| `Enter` | Toggle step on/off, then advance cursor to next step (holding Enter fills consecutive steps) |

### Drum Track Controls (FocusSection::DrumTrackControls)

| Key | Action |
|---|---|
| `Left` / `Right` | Navigate fields within current page. Left at first field returns to grid (step 31). Right at last field returns to grid (step 0). |
| `Up` / `Down` | Move to prev/next track (wraps, syncs with grid cursor) |
| `Shift+Up` / `Shift+Down` | Adjust selected parameter value (+/-0.05), or toggle Mute/Solo |
| `Alt+Up` / `Alt+Down` | Adjust value AND audition the sound (triggers voice + flash) |

### Modal Dialogs

**Text Input** (save project name):
| Key | Action |
|---|---|
| Characters | Append to buffer (max 40 chars) |
| `Backspace` | Delete last character |
| `Enter` | Confirm |
| `Esc` | Cancel |

**File Picker** (load project or kit):
| Key | Action |
|---|---|
| `Up` / `Down` | Navigate list |
| `Enter` | Load selected item |
| `Esc` | Cancel |

---

## 8. Entry Point & Event Loop (`src/main.rs`)

```rust
fn main() -> io::Result<()> {
    // 1. Create crossbeam channels
    // 2. Start audio stream (must keep _stream alive)
    // 3. Initialize terminal (raw mode, alternate screen, mouse capture)
    // 4. Create App (loads demo_project(), sends initial state to audio thread)

    // 5. Main loop:
    //    a. Check splash: if active, tick splash animation (passing terminal size),
    //       render splash, check for skip key or mouse click, sleep 16ms, continue
    //    b. app.tick()  — drain audio->UI messages, decay flashes, decay status msg,
    //                     handle queued pattern switch at step 0
    //    c. terminal.draw(|f| ui::render(f, &app))
    //    d. Drain ALL pending crossterm events (poll with 0ms timeout)
    //       - Event::Key: handle Press and Repeat via keys::handle_key()
    //       - Event::Mouse: handle via mouse::handle_mouse()
    //    e. Sleep 16ms for ~60fps
    //    f. break if app.should_quit

    // 6. Restore terminal (disable raw mode, leave alternate screen, disable mouse capture)
}
```

The event loop drains all pending input events per frame rather than blocking on a single event, ensuring responsive key repeat and no input lag during held keys. Mouse events are handled in the same drain loop.

---

## 9. Project System

### File Format (.tsp)

Projects are saved as pretty-printed JSON with the `.tsp` extension. Example structure:

```json
{
  "textstep": { "format_version": 1, "app_version": "0.1.0" },
  "metadata": { "name": "My Beat", "author": "" },
  "kits": [
    {
      "name": "Kit 1",
      "tracks": [
        { "id": "kick", "tune": 0.3, "sweep": 0.6, "color": 0.2, "snap": 0.5,
          "filter": 0.7, "drive": 0.2, "decay": 0.5, "volume": 0.8,
          "send_reverb": 0.0, "send_delay": 0.0 },
        ...
      ]
    },
    ...
  ],
  "active_kit": 0,
  "patterns": [
    { "name": "House", "bpm": 125.0, "steps": ["88880000", "00800000", ...] },
    ...
  ],
  "active_pattern": 0,
  "bpm": 125.0,
  "loop_length": 16,
  "effects": {
    "reverb_amount": 0.3, "reverb_damping": 0.4,
    "delay_time": 0.5, "delay_feedback": 0.3, "delay_tone": 0.6,
    "compressor_amount": 0.0
  }
}
```

### Demo Project (`demo_project()`)

The app starts with a pre-loaded demo project containing 10 genre patterns, each with its own BPM:

| # | Pattern | BPM | Characteristics |
|---|---------|-----|-----------------|
| 1 | House | 125 | Four-on-the-floor, 16th-note CHH/OHH, tom accent on beat 2 |
| 2 | Chicago House | 122 | Upbeat kick anticipation, clap+snare on 2&4, paired hats |
| 3 | Brit House | 122 | Syncopated kick (ghost on 7), ride accents, clap on beat 3 |
| 4 | French House | 124 | Funky syncopated kick, cowbell shakers on 8th notes |
| 5 | Dirty House | 126 | Sparse hats, offbeat claps, gritty kick syncopation |
| 6 | Trance | 138 | Four-on-the-floor, crash ride on beat 1, dense hats |
| 7 | Techno | 130 | Clean 8th-note hats, offbeat ride, clap on 2&4 |
| 8 | Drum & Bass | 170 | Breakbeat kick (0,10), snare on beat 3, dense hats |
| 9 | Trap | 140 | Hi-hat rolls (16ths→8ths), sparse kick, tom fills |
| 10 | Moombahton | 100 | Dembow-influenced, dense hat interplay, ride on beat 1 |

All patterns are 16 steps (loop_length=16). Patterns transcribed from the "Drum Pattern Cheat Sheet" by Subaqueous.

### Hex Step Encoding

Each track's 32 steps are packed as 8 hex characters (4 steps per nibble, MSB first):
- `steps_to_hex()`: `[true,false,false,false, true,false,false,false, ...]` -> `"88880000"`
- `hex_to_steps()`: reverse conversion

### Per-Pattern BPM

Each `PatternData` has an optional `bpm` field (default 0.0 via `#[serde(default)]`):
- **0.0** means "use project BPM" (backward compatible with old files)
- **Non-zero** values are applied automatically when switching patterns
- BPM is saved back to the current pattern's `bpm` field on every `store_current_to_project()` call
- On project load, the active pattern's BPM is applied to the transport

### Save/Load Flow

- **Ctrl+S**: If a path exists, saves immediately. Otherwise opens a text input modal for the project name. Name is slugified to a filename.
- **Ctrl+O**: Scans `projects_dir()` for `.tsp` files, reads their metadata names, presents a file picker sorted alphabetically.
- **Ctrl+N**: Opens text input modal to rename the current pattern.
- **Ctrl+K**: Saves the active kit as a standalone `.tsk` file. Prompts for name if kit name starts with "Kit " or is empty.
- **Ctrl+J**: Scans `kits_dir()` for `.tsk` files, presents a file picker ("Load Kit into slot N"). Loading a kit replaces the active kit slot's sound params while preserving step data.
- `ProjectFile::normalize()` ensures 10 patterns and 8 kits exist after loading. Migrates old single-kit files by seeding the first kit slot from the legacy `kit` field, then filling remaining slots with defaults.
- `store_current_to_project()` syncs live app state (steps, active kit params, BPM, loop length, effect params, per-pattern BPM) back to the `ProjectFile` before saving or switching patterns.

### Standalone Kit Files (.tsk)

Kits can be saved/loaded independently from projects as `.tsk` JSON files in `~/Library/Application Support/textstep/kits/`. A kit contains the name and all 8 tracks' sound parameters (including send levels). Loading a kit replaces the active kit slot's params while preserving mute/solo state and step patterns.

### Pattern Switching

- **Queued**: pattern change takes effect when playback wraps to step 0 (checked in `App::tick()` when `PlaybackPosition` arrives with `drum_step == 0`). Queued pattern shown with yellow background.
- **Immediate**: saves current pattern, clears steps, loads new pattern steps, sends to audio thread instantly. Applies per-pattern BPM.
- Same-pattern queue cancels the queue (toggle behavior).

---

## 10. Test Coverage

23 unit tests across 4 modules:

| Module | Tests | What's Covered |
|---|---|---|
| `audio::clock` | 7 | Step timing, wrapping, beat calc, bar start, reset, first-step-fires |
| `audio::mixer` | 6 | Mute/solo logic (none, mute-only, solo-override), soft clip passthrough/limits/symmetry |
| `sequencer::project` | 6 | Hex roundtrip, hex all-on, hex empty, default 10 patterns, serialize roundtrip, forward-compat missing fields |
| `sequencer::project::demo_tests` | 4 | Demo project has 10 patterns, House kick is four-on-the-floor, per-pattern BPM values, demo serialize roundtrip |

Run with: `cargo test`

---

## 11. Potential Future Work

- Copy/paste patterns or sections
- Swing/shuffle timing
- Per-drum-track pan (stereo field)
- Additional drum voice synthesis models
- MIDI input support
- Pattern chaining / song mode
- Undo/redo for pattern edits
- Master volume UI control with knob
- Bass synthesizer (303-style acid line)
- Global effect parameter UI controls (currently only per-track sends and compressor have UI; reverb/delay params need UI)
- Cross-platform data directory (Linux XDG support is stubbed)
