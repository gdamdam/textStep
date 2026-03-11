# Panel Minimize System & Dual Synth

Date: 2026-03-11

## Overview

Two features that work together: (1) per-panel minimize toggles that let users collapse any major UI section, reclaiming vertical space, and (2) a second independent synth (Synth A / Synth B) that doubles the melodic capability.

The minimize system is a prerequisite for the dual synth — without it, two full synth sections would overflow most terminal heights.

## Feature 1: Panel Minimize System

### Behavior

Every major panel gets a clickable `[X]` toggle rendered in its top-right border area:
- **Lit** (amber `[X]`) — panel is visible
- **Dim** (dark gray `[.]`) — panel is collapsed

Panels that get the toggle:
- Synth A Knobs
- Synth A Grid
- Synth B Knobs
- Synth B Grid
- Drum Grid
- Drum Knobs
- Waveform (when active)

**Transport bar does NOT get a toggle** — always visible.

### Collapsed State

A collapsed panel renders as a minimal ratatui Block with top+bottom borders, title in top border:

```
+-- SYNTH A KNOBS --------------------------------- [.] --+
+----------------------------------------------------------+
```

Height: 2 lines (top border with title + bottom border, no content). Constant: `COLLAPSED_PANEL_HEIGHT = 2`.

### Space Reclamation

When a panel is collapsed, adjacent panels expand to fill the reclaimed space. Collapsed panels use `Constraint::Length(COLLAPSED_PANEL_HEIGHT)` (2 lines); the drum grid area uses `Constraint::Min(11)` to absorb freed space. If multiple panels are collapsed, the `Min` constraint panel grows proportionally.

### State

`UiState` gains a `PanelVisibility` struct:

```rust
struct PanelVisibility {
    synth_a_knobs: bool,  // default true
    synth_a_grid: bool,   // default true
    synth_b_knobs: bool,  // default false (collapsed by default to fit terminals)
    synth_b_grid: bool,   // default false
    drum_grid: bool,      // default true
    drum_knobs: bool,     // default true
    waveform: bool,       // default true (replaces UiState.show_waveform)
}
```

Note: `PanelVisibility.waveform` replaces the existing `UiState.show_waveform` field. The old field is removed to avoid duplication.

### Interaction

- **Mouse**: clicking the `[X]`/`[.]` region toggles visibility
- **Keyboard**: F2 is retained as a bulk toggle — collapses/expands all synth panels (A knobs + A grid + B knobs + B grid) at once. This preserves keyboard-only usability.

### Replaces F2 Single Toggle

The current F2 behavior (single synth collapse) is replaced by the bulk synth toggle described above. Individual panel [X] toggles are mouse-only initially.

### Mouse Hit-Testing

`mouse.rs` must detect clicks on the `[X]`/`[.]` regions. These are always rendered at a known offset from the right edge of each panel's title bar, making hit-testing straightforward: check if click is within the last ~4 columns of a panel's top border row.

## Feature 2: Dual Synth (Synth A / Synth B)

### Data Model

**App state** (`app.rs`):
- `synth_a_pattern: SynthPattern` and `synth_b_pattern: SynthPattern`
- All synth-related `UiState` fields duplicated per instance:
  - `synth_a_active_pattern`, `synth_b_active_pattern`
  - `synth_a_queued_pattern`, `synth_b_queued_pattern`
  - `synth_a_active_kit`, `synth_b_active_kit`
  - `synth_a_cursor_step`, `synth_b_cursor_step`
  - `synth_a_ctrl_field`, `synth_b_ctrl_field`

**FocusSection** gains new variants:
- `SynthAGrid`, `SynthAControls` (replacing current `SynthGrid`, `SynthControls`)
- `SynthBGrid`, `SynthBControls`

### Project Storage

`ProjectFile` (`project.rs`) gains:
- `synth_b_patterns: Vec<SynthPatternData>` (10 patterns)
- `synth_b_kits: Vec<SynthKitData>` (8 kits)
- `active_synth_b_pattern: usize`
- `active_synth_b_kit: usize`

Each synth has fully independent pattern and kit banks.

### Messages

`UiToAudio` / `AudioToUi` messages split per synth:
- `SetSynthAPattern(SynthPattern)` / `SetSynthBPattern(SynthPattern)`
- `SetSynthAKit(...)` / `SetSynthBKit(...)`
- Or parameterized: `SetSynthPattern(SynthId, SynthPattern)` where `SynthId` is `A` or `B`

Parameterized form is preferred to avoid doubling every message variant.

### Audio Engine

**Voices**: Two `SynthVoice` instances, two independent LFO structs.

**Effects**: Each synth gets its own reverb and delay **instances** with independently configurable parameters. Three fully independent FX chains (Synth A, Synth B, Drums). Each chain has:
- Per-voice **send levels** (`send_reverb`, `send_delay`) — already exist in `SynthParams`
- Per-chain **effect character** (reverb: amount + damping; delay: time + feedback + tone) — new per-synth fields

The current global `EffectParams` struct is split: compressor and master volume remain global; reverb/delay params move into each `SynthInstance` (see below). CPU note: 3x reverb instances is acceptable — the reverb is a simple Schroeder design, not convolution.

**Mixer path**:
```
synth_a_voice -> synth_a_saturator -> synth_a_reverb/delay -> synth_a_out
synth_b_voice -> synth_b_saturator -> synth_b_reverb/delay -> synth_b_out
drum_voices  -> drum_saturator    -> drum_reverb/delay    -> drum_out

(synth_a_out + synth_b_out + drum_out) * master_vol -> compressor -> clip
```

**Clock**: Both synths share the global clock but have independent loop lengths via `transport.loop_config`.

### UI Layout

Stacked vertical layout:

```
+-- TRANSPORT (7h) ----------------------------------------+
| [>] BPM:120  @@@@.  Swing:50%  *REC   |VU|  SAT         |
| Synth A [3/q] Kit: Pad     Loop ON  S:16                 |
| Synth B [5/.] Kit: Lead    Loop ON  S:8                  |
| Drum    [1/.] Kit: HipHop  Loop ON  D:32                 |
+-- SYNTH A KNOBS --------------------------------- [X] ---+
| OSC1 | OSC2 | ENV1 | ENV2 | FILT | LFO | AMP            |
+-- SYNTH A GRID ---------------------------------- [X] ---+
| step grid (32 steps)                                      |
+-- SYNTH B KNOBS --------------------------------- [X] ---+
| OSC1 | OSC2 | ENV1 | ENV2 | FILT | LFO | AMP            |
+-- SYNTH B GRID ---------------------------------- [X] ---+
| step grid (32 steps)                                      |
+----------------------------------------------------------+  <-- separator (between synths and drums)
+-- DRUM GRID ------------------------------------- [X] ---+
| 8 tracks x 32 steps                                      |
+-- DRUM KNOBS ------------------------------------ [X] ---+
| Tune Sweep Color Snap Filt Drv Dec Vol Rv Dl             |
+----------------------------------------------------------+
```

`TRANSPORT_HEIGHT` increases from 6 to 7 (one extra line for Synth B status).

No separator between Synth A and Synth B sections — they are visually distinguished by their titled borders (`SYNTH A KNOBS` / `SYNTH B KNOBS`). The single separator line remains between the synth group and the drum group.

### Navigation

**Tab** cycles focus through all sections in order:
Transport -> Synth A Controls -> Synth A Grid -> Synth B Controls -> Synth B Grid -> Drum Grid -> Drum Knobs -> (wrap)

Collapsed panels are skipped in the Tab cycle.

### Focus-Aware Key Bindings

These keys apply to whichever synth (or drums) is currently focused:
- `1-0` — select pattern 1-10
- `Shift+K` — cycle kit
- `L` — cycle loop length
- Arrow keys — navigate params/steps within the focused section

### Presets

Both synths share the same preset banks (synth presets, synth pattern presets). Each synth independently browses and loads presets.

## Layout Redesign: `compute_layout()`

The current `compute_layout()` takes `synth_collapsed: bool` and returns a flat `ComputedLayout`. This must be redesigned for per-panel collapse and dual synths.

### New `ComputedLayout` Struct

```rust
pub struct ComputedLayout {
    pub transport: Rect,
    // Synth A
    pub synth_a_knobs: Rect,    // full or collapsed
    pub synth_a_grid: Rect,     // full or collapsed
    pub synth_a_fader: Rect,    // volume fader (zero-width when knobs collapsed)
    // Synth B
    pub synth_b_knobs: Rect,
    pub synth_b_grid: Rect,
    pub synth_b_fader: Rect,
    // Separator
    pub separator: Rect,
    // Drums
    pub drum_grid: Rect,        // full or collapsed
    pub drum_knobs: Rect,       // full or collapsed
    pub drum_fader: Rect,
    // Extras
    pub extra: Option<Rect>,    // Help or Waveform
    pub activity_bar: Rect,
}
```

### New `compute_layout()` Signature

```rust
pub fn compute_layout(
    size: Rect,
    panel_vis: &PanelVisibility,
    show_help: bool,
    // note: waveform visibility comes from panel_vis.waveform, not a separate param
) -> ComputedLayout
```

### Constraint Strategy

Each panel contributes either its full height or `COLLAPSED_PANEL_HEIGHT` (2):

```rust
let sa_knobs_h = if panel_vis.synth_a_knobs { SYNTH_KNOBS_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
let sa_grid_h  = if panel_vis.synth_a_grid  { SYNTH_GRID_HEIGHT }  else { COLLAPSED_PANEL_HEIGHT };
let sb_knobs_h = if panel_vis.synth_b_knobs { SYNTH_KNOBS_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
let sb_grid_h  = if panel_vis.synth_b_grid  { SYNTH_GRID_HEIGHT }  else { COLLAPSED_PANEL_HEIGHT };
let drum_knobs_h = if panel_vis.drum_knobs  { KNOBS_HEIGHT }       else { COLLAPSED_PANEL_HEIGHT };

// Flex element: drum grid absorbs reclaimed space when visible.
// When drum grid is collapsed, the activity_bar area becomes the flex element.
let drum_grid_h = if panel_vis.drum_grid { Constraint::Min(11) } else { Constraint::Length(COLLAPSED_PANEL_HEIGHT) };
let activity_h  = if panel_vis.drum_grid { Constraint::Length(ACTIVITY_BAR_HEIGHT) } else { Constraint::Min(1) };
```

The drum grid is the primary flex element. If drum grid itself is collapsed, the activity bar area absorbs freed space (acts as a spacer).

Volume faders are only rendered when the corresponding knobs panel is visible. When collapsed, the fader rect has zero width.

## Layout Constants Changes

In `layout.rs`:
- `TRANSPORT_HEIGHT`: 6 -> 7
- `COLLAPSED_PANEL_HEIGHT`: new constant = 2
- `SYNTH_COLLAPSED_HEIGHT`: removed (replaced by generic collapse)
- `SYNTH_SECTION_HEIGHT`: removed (each sub-panel sized independently)

## Audio Engine Duplication Detail

### SynthInstance Struct

To avoid duplicating 8+ fields with `_a`/`_b` suffixes, introduce a `SynthInstance` struct that bundles all per-synth audio state:

```rust
pub struct SynthInstance {
    pub pattern: SynthPattern,
    pub voice: SynthVoice,
    pub gate_samples: u32,
    pub note_end_step: Option<usize>,
    pub lfo: Lfo,
    pub saturator: TubeSaturator,
    pub reverb: ReverbEffect,
    pub delay: DelayEffect,
    // Per-instance effect character params
    pub reverb_amount: f32,
    pub reverb_damping: f32,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_tone: f32,
}
```

AudioEngine then holds:

```rust
pub synth_a: SynthInstance,
pub synth_b: SynthInstance,
```

This pattern also applies to `UiState` (a `SynthUiState` struct) and `ProjectFile` (a `SynthProjectData` struct) to reduce field duplication throughout.

The global `EffectParams` struct is simplified to only master-bus params: `drum_volume`, `master_volume`, `compressor_amount`. Per-synth effect params live inside each `SynthInstance`.

### AudioToUi::PlaybackPosition Update

```rust
PlaybackPosition {
    global_step: usize,
    beat: u8,
    is_bar_start: bool,
    triggered: u8,              // drum bitmask (unchanged)
    synth_a_triggered: bool,    // was: synth_triggered
    synth_b_triggered: bool,    // new
    drum_step: usize,           // unchanged
    synth_a_step: usize,        // was: synth_step
    synth_b_step: usize,        // new
}
```

### Transport Loop Config

`LoopConfig` gains `synth_b_length: usize` alongside existing `synth_length` (renamed to `synth_a_length`) and `drum_length`.

The rename requires `#[serde(alias = "synth_length")]` on `synth_a_length` for backward compatibility with existing `.tsp` files.

Affected call sites for the rename:
- `keys.rs` — loop length cycling logic
- `clock.rs` — step computation (`global_step % synth_length`)
- `engine.rs` — synth step modulo in the audio callback
- `transport_bar.rs` — loop length display

## FocusSection Navigation with Collapsed Panels

`FocusSection::next()` / `prev()` must skip collapsed panels. The implementation takes `&PanelVisibility` as a parameter and advances past invisible panels. Safety: if ALL collapsible panels are collapsed, focus stays on Transport (which is always visible) to prevent infinite loops.

```rust
impl FocusSection {
    pub fn next(&self, vis: &PanelVisibility) -> FocusSection {
        let order = [Transport, SynthAControls, SynthAGrid, SynthBControls,
                     SynthBGrid, DrumGrid, Knobs];
        // find current index, advance, skip invisible, wrap
        // if full cycle with no visible panel found, return Transport
    }
}
```

## Testing Strategy

- Unit tests for `PanelVisibility` state toggling
- Unit tests for layout computation with various collapse combinations
- Unit tests for dual synth pattern/kit independence
- Integration test: project save/load roundtrip with dual synth data
- Manual testing: verify mouse hit-testing matches render for all collapse states

## Terminal Size & Default State

With all panels expanded, the minimum terminal height is ~101 rows (2x30 synth knobs + 2x6 synth grids + 7 transport + 11 drum grid + 9 drum knobs + 1 separator + 1 activity bar). This exceeds most terminals (typical: 24-50 rows).

**Default state**: Synth B panels start collapsed (`synth_b_knobs: false`, `synth_b_grid: false`). This brings the default to ~65 rows (still tall, but manageable at standard terminal sizes with some panels auto-fitting). Users expand Synth B panels on demand.

No auto-collapse based on terminal size — the user controls visibility explicitly.

## Migration

- Existing `.tsp` project files lack Synth B data. On load, Synth B initializes with defaults (empty patterns, default kit). Backward-compatible: old files load fine, new fields get defaults via `#[serde(default)]`.
- `LoopConfig.synth_length` renamed to `synth_a_length` with `#[serde(alias = "synth_length")]` for backward compatibility.
