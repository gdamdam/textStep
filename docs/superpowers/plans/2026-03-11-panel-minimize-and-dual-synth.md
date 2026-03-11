# Panel Minimize & Dual Synth Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add per-panel [X] minimize toggles to all major UI panels and a second independent synth (Synth A / Synth B) with full FX chains.

**Architecture:** Two interleaved features sharing the same layout redesign. Panel minimize provides the vertical space needed for dual synth. Data model changes come first, then audio engine and layout in parallel, then UI integration on top.

**Tech Stack:** Rust, ratatui, cpal, crossbeam, serde

**Spec:** `docs/superpowers/specs/2026-03-11-panel-minimize-and-dual-synth-design.md`

---

## File Map

**Create:**
- (none — all changes are modifications to existing files)

**Modify:**
- `src/app.rs` — PanelVisibility, SynthUiState, FocusSection, App struct
- `src/messages.rs` — SynthId enum, parameterized messages, PlaybackPosition
- `src/sequencer/transport.rs` — LoopConfig synth_a/b_length
- `src/sequencer/synth_pattern.rs` — (no structural changes, used as-is)
- `src/sequencer/project.rs` — ProjectFile synth B storage, migration
- `src/audio/engine.rs` — SynthInstance struct, dual synth processing, dual FX
- `src/audio/effects.rs` — (no changes, types used as-is)
- `src/audio/synth_voice.rs` — (no changes, instantiated twice)
- `src/ui/layout.rs` — constants, ComputedLayout, compute_layout()
- `src/ui/mod.rs` — render dispatch, panel collapse, dual synth sections
- `src/ui/transport_bar.rs` — 3 status lines (Synth A, Synth B, Drum)
- `src/ui/synth_grid.rs` — parameterize for synth A/B
- `src/ui/synth_knobs.rs` — parameterize for synth A/B
- `src/ui/knobs.rs` — add [X] toggle to drum knobs title
- `src/ui/waveform.rs` — add [X] toggle to waveform title
- `src/mouse.rs` — [X] hit-testing, dual synth hit-testing, layout update
- `src/keys.rs` — F2 bulk toggle, focus-aware dual synth keys, FocusSection update
- `src/main.rs` — (minor: pass new params to audio engine constructor)
- `src/params.rs` — EffectParams simplification (if needed)
- `src/presets/synth_presets.rs` — synth B preset loading
- `src/presets/synth_pattern_presets.rs` — synth B pattern preset loading
- `src/presets/mod.rs` — preset browser synth B support

---

## Chunk 1: Data Model Foundation

All subsequent chunks depend on this. Changes must compile but don't need to be wired into UI/audio yet.

### Task 1: PanelVisibility Struct

**Files:**
- Modify: `src/app.rs` (after line 435, near `synth_collapsed: bool`)

- [ ] **Step 1: Add PanelVisibility struct to app.rs**

Add after the existing `UiState` field declarations (around line 49, after imports):

```rust
#[derive(Clone, Debug)]
pub struct PanelVisibility {
    pub synth_a_knobs: bool,
    pub synth_a_grid: bool,
    pub synth_b_knobs: bool,
    pub synth_b_grid: bool,
    pub drum_grid: bool,
    pub drum_knobs: bool,
    pub waveform: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            synth_a_knobs: true,
            synth_a_grid: true,
            synth_b_knobs: false,  // collapsed by default
            synth_b_grid: false,   // collapsed by default
            drum_grid: true,
            drum_knobs: true,
            waveform: true,
        }
    }
}
```

- [ ] **Step 2: Replace synth_collapsed and show_waveform in UiState**

In `UiState` struct (line ~435):
- Remove field `show_waveform: bool` (line 434)
- Remove field `synth_collapsed: bool` (line 435)
- Add field `panel_vis: PanelVisibility`

In `Default for UiState` (line ~486):
- Remove `show_waveform: false` and `synth_collapsed: false`
- Add `panel_vis: PanelVisibility::default()`

- [ ] **Step 3: Add compatibility shims (temporary)**

Anywhere that reads `app.ui.synth_collapsed`, temporarily replace with `!app.ui.panel_vis.synth_a_knobs || !app.ui.panel_vis.synth_a_grid`. Anywhere that reads `app.ui.show_waveform`, replace with `app.ui.panel_vis.waveform`. This keeps the build green while we refactor.

Use `cargo build` to find all references and fix them.

- [ ] **Step 4: Build and verify**

Run: `cargo build 2>&1 | head -50`
Expected: Compiles successfully (with warnings OK)

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/ui/mod.rs src/mouse.rs src/keys.rs
git commit -m "refactor: add PanelVisibility, replace synth_collapsed and show_waveform"
```

---

### Task 2: SynthId Enum and Message Updates

**Files:**
- Modify: `src/messages.rs`

- [ ] **Step 1: Add SynthId enum**

Add before the `UiToAudio` enum (line ~7):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SynthId {
    A,
    B,
}
```

- [ ] **Step 2: Parameterize synth messages**

Update `UiToAudio` (lines 8-16):

```rust
pub enum UiToAudio {
    SetTransport(Transport),
    SetDrumPattern(DrumPattern),
    SetSynthPattern(SynthId, SynthPattern),  // was: SetSynthPattern(SynthPattern)
    SetEffectParams(EffectParams),
    TriggerDrum(DrumTrackId),
    TriggerSynth(SynthId, u8),               // was: TriggerSynth(u8)
    ReleaseSynth(SynthId),                    // was: ReleaseSynth
}
```

- [ ] **Step 3: Update PlaybackPosition**

Update `AudioToUi::PlaybackPosition` (lines 18-28):

```rust
pub enum AudioToUi {
    PlaybackPosition {
        global_step: usize,
        beat: u8,
        is_bar_start: bool,
        triggered: u8,
        synth_a_triggered: bool,   // was: synth_triggered
        synth_b_triggered: bool,   // new
        drum_step: usize,
        synth_a_step: usize,       // was: synth_step
        synth_b_step: usize,       // new
    },
}
```

- [ ] **Step 4: Fix all compilation errors from message changes**

Run `cargo build` and fix every call site:
- `src/audio/engine.rs` — sends PlaybackPosition, receives SetSynthPattern/TriggerSynth/ReleaseSynth
- `src/app.rs` or `src/main.rs` — sends SetSynthPattern, TriggerSynth, ReleaseSynth
- `src/keys.rs` — sends TriggerSynth, ReleaseSynth, SetSynthPattern
- `src/mouse.rs` — may send SetSynthPattern

For now, route all synth messages to `SynthId::A` to keep behavior identical. Synth B wiring comes later.

- [ ] **Step 5: Build and test**

Run: `cargo build && cargo test`
Expected: All compile, all 23 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/messages.rs src/audio/engine.rs src/app.rs src/keys.rs src/mouse.rs src/main.rs
git commit -m "refactor: add SynthId, parameterize synth messages"
```

---

### Task 3: FocusSection and SynthUiState

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add SynthUiState struct**

Add near `PanelVisibility` in `app.rs`:

```rust
#[derive(Clone, Debug)]
pub struct SynthUiState {
    pub playback_step: usize,
    pub cursor_step: usize,
    pub ctrl_field: SynthControlField,
    pub flash: u8,
    pub octave: u8,
    pub active_pattern: usize,
    pub queued_pattern: Option<usize>,
    pub active_kit: usize,
}

impl Default for SynthUiState {
    fn default() -> Self {
        Self {
            playback_step: 0,
            cursor_step: 0,
            ctrl_field: SynthControlField::Osc1Waveform,
            flash: 0,
            octave: 4,
            active_pattern: 0,
            queued_pattern: None,
            active_kit: 0,
        }
    }
}
```

- [ ] **Step 2: Update FocusSection enum**

Replace the `FocusSection` enum (lines 21-49):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    DrumGrid,
    Knobs,
    SynthAGrid,      // was: SynthGrid
    SynthAControls,  // was: SynthControls
    SynthBGrid,      // new
    SynthBControls,  // new
    Transport,
}

impl FocusSection {
    pub fn next(&self, vis: &PanelVisibility) -> Self {
        use FocusSection::*;
        let order = [
            Transport, SynthAControls, SynthAGrid,
            SynthBControls, SynthBGrid, DrumGrid, Knobs,
        ];
        let cur = order.iter().position(|s| s == self).unwrap_or(0);
        for i in 1..=order.len() {
            let candidate = order[(cur + i) % order.len()];
            if candidate.is_visible(vis) {
                return candidate;
            }
        }
        Transport // fallback: transport is always visible
    }

    pub fn prev(&self, vis: &PanelVisibility) -> Self {
        use FocusSection::*;
        let order = [
            Transport, SynthAControls, SynthAGrid,
            SynthBControls, SynthBGrid, DrumGrid, Knobs,
        ];
        let cur = order.iter().position(|s| s == self).unwrap_or(0);
        for i in 1..=order.len() {
            let candidate = order[(cur + order.len() - i) % order.len()];
            if candidate.is_visible(vis) {
                return candidate;
            }
        }
        Transport
    }

    pub fn is_visible(&self, vis: &PanelVisibility) -> bool {
        use FocusSection::*;
        match self {
            Transport => true,
            SynthAControls => vis.synth_a_knobs,
            SynthAGrid => vis.synth_a_grid,
            SynthBControls => vis.synth_b_knobs,
            SynthBGrid => vis.synth_b_grid,
            DrumGrid => vis.drum_grid,
            Knobs => vis.drum_knobs,
        }
    }
}
```

- [ ] **Step 3: Replace individual synth fields in UiState with SynthUiState**

In `UiState` struct, replace all `synth_*` fields (lines 455-467) with:

```rust
pub synth_a: SynthUiState,
pub synth_b: SynthUiState,
```

In `Default for UiState`, replace synth field defaults with:

```rust
synth_a: SynthUiState::default(),
synth_b: SynthUiState::default(),
```

- [ ] **Step 4: Fix all references to old synth UiState fields**

Run `cargo build` and fix every reference. The mapping is:
- `app.ui.synth_playback_step` -> `app.ui.synth_a.playback_step`
- `app.ui.synth_cursor_step` -> `app.ui.synth_a.cursor_step`
- `app.ui.synth_ctrl_field` -> `app.ui.synth_a.ctrl_field`
- `app.ui.synth_flash` -> `app.ui.synth_a.flash`
- `app.ui.synth_octave` -> `app.ui.synth_a.octave`
- `app.ui.synth_active_pattern` -> `app.ui.synth_a.active_pattern`
- `app.ui.synth_queued_pattern` -> `app.ui.synth_a.queued_pattern`
- `app.ui.synth_active_kit` -> `app.ui.synth_a.active_kit`
- `FocusSection::SynthGrid` -> `FocusSection::SynthAGrid`
- `FocusSection::SynthControls` -> `FocusSection::SynthAControls`

Also fix `FocusSection::next()`/`prev()` call sites — they now take `&app.ui.panel_vis`:
- `app.ui.focus.next()` -> `app.ui.focus.next(&app.ui.panel_vis)`
- `app.ui.focus.prev()` -> `app.ui.focus.prev(&app.ui.panel_vis)`

- [ ] **Step 5: Update App struct for dual synth patterns**

In `App` struct (line ~525), replace:
- `synth_pattern: SynthPattern` with `synth_a_pattern: SynthPattern` and `synth_b_pattern: SynthPattern`

Fix all references: `app.synth_pattern` -> `app.synth_a_pattern`. Synth B pattern initializes as `SynthPattern::default()`.

- [ ] **Step 6: Build and test**

Run: `cargo build && cargo test`
Expected: All compile, all tests pass

- [ ] **Step 7: Commit**

```bash
git add src/app.rs src/keys.rs src/mouse.rs src/ui/
git commit -m "refactor: SynthUiState, dual FocusSection, dual synth patterns in App"
```

---

### Task 4: LoopConfig Update

**Files:**
- Modify: `src/sequencer/transport.rs`

- [ ] **Step 1: Add synth_b_length to LoopConfig**

Update `LoopConfig` struct (lines 17-21). Verify it already derives `Serialize, Deserialize` (needed for serde attributes). If not, add the derives:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LoopConfig {
    pub enabled: bool,
    pub drum_length: u8,
    #[serde(alias = "synth_length")]
    pub synth_a_length: u8,    // was: synth_length
    #[serde(default = "default_synth_b_length")]
    pub synth_b_length: u8,    // new
}

fn default_synth_b_length() -> u8 { 16 }
```

Note: If `LoopConfig` does not derive `Serialize`/`Deserialize`, add the derives. Check `Transport` too — it embeds `LoopConfig` and is serialized as part of project files.

- [ ] **Step 2: Fix all references to synth_length**

Run `cargo build` and fix:
- `src/keys.rs` — loop length cycling: `loop_config.synth_length` -> `loop_config.synth_a_length` (for synth A focused) or `loop_config.synth_b_length` (for synth B focused)
- `src/audio/engine.rs` — `synth_loop_len` computation: use synth_a_length and synth_b_length
- `src/ui/transport_bar.rs` — display: show both lengths
- `src/sequencer/project.rs` — any save/load referencing synth_length

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`
Expected: All pass. Serde alias ensures old files still load.

- [ ] **Step 4: Commit**

```bash
git add src/sequencer/transport.rs src/keys.rs src/audio/engine.rs src/ui/transport_bar.rs src/sequencer/project.rs
git commit -m "refactor: LoopConfig synth_a_length + synth_b_length with serde alias"
```

---

## Chunk 2: Audio Engine — Dual Synth

Depends on Chunk 1. Can be developed in parallel with Chunk 3 (Layout).

### Task 5: SynthInstance Struct in Audio Engine

**Files:**
- Modify: `src/audio/engine.rs`

- [ ] **Step 1: Define SynthInstance struct**

Add before `AudioEngine` struct (line ~78):

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
    // Per-instance effect params
    pub reverb_amount: f32,
    pub reverb_damping: f32,
    pub delay_time: f32,
    pub delay_feedback: f32,
    pub delay_tone: f32,
}

impl SynthInstance {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            pattern: SynthPattern::default(),
            voice: SynthVoice::new(sample_rate as f32),
            gate_samples: 0,
            note_end_step: None,
            lfo: Lfo::new(),  // Lfo has new() not Default
            saturator: TubeSaturator::new(sample_rate as f32),
            reverb: ReverbEffect::new(sample_rate),
            delay: DelayEffect::new(),
            reverb_amount: 0.3,
            reverb_damping: 0.5,
            delay_time: 0.0,
            delay_feedback: 0.4,
            delay_tone: 0.5,
        }
    }
}
```

- [ ] **Step 2: Replace individual synth fields in AudioEngine**

In `AudioEngine` struct (lines 79-113), replace these fields:
```
synth_pattern: SynthPattern        -> (removed)
synth_voice: SynthVoice            -> (removed)
synth_gate_samples: u32            -> (removed)
synth_note_end_step: Option<usize> -> (removed)
lfo: Lfo                           -> (removed)
synth_saturator: TubeSaturator     -> (removed)
```

Add:
```rust
synth_a: SynthInstance,
synth_b: SynthInstance,
```

Rename the existing shared `reverb` and `delay` fields to `drum_reverb` and `drum_delay`. These become the drum FX chain. Each SynthInstance carries its own reverb/delay. Also rename `synth_saturator` -> remove (now in SynthInstance), keep `drum_saturator` as-is.

- [ ] **Step 3: Update AudioEngine::new()**

In `new()` (lines 116-156), replace individual synth field initialization with:
```rust
synth_a: SynthInstance::new(sample_rate),
synth_b: SynthInstance::new(sample_rate),
```

- [ ] **Step 4: Update message handling in process()**

In the message drain loop, update handlers:
- `UiToAudio::SetSynthPattern(id, pat)` -> match on `id`: `SynthId::A => self.synth_a.pattern = pat`, `SynthId::B => self.synth_b.pattern = pat`
- `UiToAudio::TriggerSynth(id, note)` -> match and trigger appropriate voice
- `UiToAudio::ReleaseSynth(id)` -> match and release appropriate voice

- [ ] **Step 5: Build (will have errors in process loop — fix in next task)**

Run: `cargo build 2>&1 | head -80`
Fix any remaining references to old field names (`self.synth_pattern` -> `self.synth_a.pattern`, etc.)

- [ ] **Step 6: Commit**

```bash
git add src/audio/engine.rs
git commit -m "refactor: SynthInstance struct, dual synth in AudioEngine"
```

---

### Task 6: Dual Synth Processing and Mixing

**Files:**
- Modify: `src/audio/engine.rs`

- [ ] **Step 1: Update synth step triggering**

In the clock tick handler (around lines 260-300), the current code does:
```rust
let synth_step = event.global_step % synth_loop_len.max(1);
```

Duplicate for both synths:
```rust
let synth_a_step = event.global_step % self.synth_a.pattern.loop_len().max(1);
let synth_b_step = event.global_step % self.synth_b.pattern.loop_len().max(1);
```

Then duplicate the synth triggering logic: check `synth_a.pattern.steps[synth_a_step]` and `synth_b.pattern.steps[synth_b_step]` independently. Each triggers its own `voice`, updates its own `gate_samples` and `note_end_step`.

- [ ] **Step 2: Update synth voice processing in the sample loop**

In the per-sample processing (where `synth_voice.tick()` is called), duplicate:
```rust
let synth_a_sample = self.synth_a.voice.tick(&self.synth_a.pattern.params, &mut self.synth_a.lfo, ...);
let synth_b_sample = self.synth_b.voice.tick(&self.synth_b.pattern.params, &mut self.synth_b.lfo, ...);
```

- [ ] **Step 3: Update FX processing**

Each synth runs through its own saturator, reverb, and delay:
```rust
// Synth A FX
let sa_sat = self.synth_a.saturator.process(synth_a_sample);
let sa_reverb = self.synth_a.reverb.tick(sa_sat * self.synth_a.pattern.params.send_reverb);
let sa_delay = self.synth_a.delay.tick(sa_sat * self.synth_a.pattern.params.send_delay);
let synth_a_out = sa_sat + sa_reverb + sa_delay;

// Synth B FX (identical structure)
let sb_sat = self.synth_b.saturator.process(synth_b_sample);
let sb_reverb = self.synth_b.reverb.tick(sb_sat * self.synth_b.pattern.params.send_reverb);
let sb_delay = self.synth_b.delay.tick(sb_sat * self.synth_b.pattern.params.send_delay);
let synth_b_out = sb_sat + sb_reverb + sb_delay;
```

- [ ] **Step 4: Update final mix**

Replace the current mono synth mix with:
```rust
let mix_l = drum_out_l + synth_a_out + synth_b_out;
let mix_r = drum_out_r + synth_a_out + synth_b_out;
// Both synths centered (mono to both channels)
```

- [ ] **Step 5: Update PlaybackPosition emission**

Update the `AudioToUi::PlaybackPosition` send to include both synth steps and triggers:
```rust
tx.send(AudioToUi::PlaybackPosition {
    global_step,
    beat,
    is_bar_start,
    triggered,
    synth_a_triggered,  // was: synth_triggered
    synth_b_triggered,  // new
    drum_step,
    synth_a_step,       // was: synth_step
    synth_b_step,       // new
}).ok();
```

- [ ] **Step 6: Update gate management**

Duplicate the synth gate countdown logic for both instances. Each `SynthInstance` manages its own `gate_samples` decrement and voice release.

- [ ] **Step 7: Build and test**

Run: `cargo build && cargo test`
Expected: Compiles. Tests pass. Synth A plays as before, Synth B is silent (empty pattern).

- [ ] **Step 8: Commit**

```bash
git add src/audio/engine.rs
git commit -m "feat: dual synth processing, independent FX chains, dual mixing"
```

---

## Chunk 3: Layout Redesign

Depends on Chunk 1. Can be developed in parallel with Chunk 2.

### Task 7: Layout Constants and ComputedLayout

**Files:**
- Modify: `src/ui/layout.rs`

- [ ] **Step 1: Update constants**

In `layout.rs` (lines 1-37):
- Change `TRANSPORT_HEIGHT` from `6` to `7`
- Add `pub const COLLAPSED_PANEL_HEIGHT: u16 = 2;`
- Remove `SYNTH_COLLAPSED_HEIGHT` (line 21)
- Remove `SYNTH_SECTION_HEIGHT` (line 18)

- [ ] **Step 2: Replace ComputedLayout struct**

Replace the entire `ComputedLayout` struct (lines 39-55) with:

```rust
pub struct ComputedLayout {
    pub transport: Rect,
    // Synth A
    pub synth_a_knobs: Rect,
    pub synth_a_grid: Rect,
    pub synth_a_fader: Rect,
    // Synth B
    pub synth_b_knobs: Rect,
    pub synth_b_grid: Rect,
    pub synth_b_fader: Rect,
    // Separator
    pub separator: Rect,
    // Drums
    pub drum_grid: Rect,
    pub drum_knobs: Rect,
    pub drum_fader: Rect,
    // Extras
    pub extra: Option<Rect>,
    pub activity_bar: Rect,
}
```

- [ ] **Step 3: Rewrite compute_layout()**

Replace the entire `compute_layout()` function (lines 58-150):

```rust
pub fn compute_layout(
    size: Rect,
    panel_vis: &crate::app::PanelVisibility,
    show_help: bool,
) -> ComputedLayout {
    let sa_knobs_h = if panel_vis.synth_a_knobs { SYNTH_KNOBS_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
    let sa_grid_h = if panel_vis.synth_a_grid { SYNTH_GRID_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
    let sb_knobs_h = if panel_vis.synth_b_knobs { SYNTH_KNOBS_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
    let sb_grid_h = if panel_vis.synth_b_grid { SYNTH_GRID_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };
    let drum_knobs_h = if panel_vis.drum_knobs { KNOBS_HEIGHT } else { COLLAPSED_PANEL_HEIGHT };

    let drum_grid_c = if panel_vis.drum_grid { Constraint::Min(11) } else { Constraint::Length(COLLAPSED_PANEL_HEIGHT) };
    let activity_c = if panel_vis.drum_grid { Constraint::Length(ACTIVITY_BAR_HEIGHT) } else { Constraint::Min(1) };

    let extra_c = if show_help {
        Some(Constraint::Length(HELP_HEIGHT))
    } else if panel_vis.waveform {
        Some(Constraint::Length(WAVEFORM_HEIGHT))
    } else {
        None
    };

    let mut constraints = vec![
        Constraint::Length(TRANSPORT_HEIGHT),      // 0: transport
        Constraint::Length(sa_knobs_h),            // 1: synth A knobs
        Constraint::Length(sa_grid_h),             // 2: synth A grid
        Constraint::Length(sb_knobs_h),            // 3: synth B knobs
        Constraint::Length(sb_grid_h),             // 4: synth B grid
        Constraint::Length(SEPARATOR_HEIGHT),      // 5: separator
        drum_grid_c,                               // 6: drum grid
        Constraint::Length(drum_knobs_h),          // 7: drum knobs
    ];

    if let Some(c) = extra_c {
        constraints.push(c);                       // 8: help or waveform
    }
    constraints.push(activity_c);                  // 8 or 9: activity bar

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(size);

    let extra_idx = if extra_c.is_some() { Some(8) } else { None };
    let activity_idx = if extra_c.is_some() { 9 } else { 8 };

    // Fader sub-splits (only when knobs are visible)
    let (sa_fader, sa_knobs_inner) = if panel_vis.synth_a_knobs {
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(FADER_WIDTH), Constraint::Min(20)])
            .split(chunks[1]);
        (h[0], h[1])
    } else {
        (Rect::default(), chunks[1])
    };

    let (sb_fader, sb_knobs_inner) = if panel_vis.synth_b_knobs {
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(FADER_WIDTH), Constraint::Min(20)])
            .split(chunks[3]);
        (h[0], h[1])
    } else {
        (Rect::default(), chunks[3])
    };

    let (drum_fader, drum_grid_inner) = if panel_vis.drum_grid {
        let h = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(FADER_WIDTH), Constraint::Min(20)])
            .split(chunks[6]);
        (h[0], h[1])
    } else {
        (Rect::default(), chunks[6])
    };

    ComputedLayout {
        transport: chunks[0],
        synth_a_knobs: sa_knobs_inner,
        synth_a_grid: chunks[2],
        synth_a_fader: sa_fader,
        synth_b_knobs: sb_knobs_inner,
        synth_b_grid: chunks[4],
        synth_b_fader: sb_fader,
        separator: chunks[5],
        drum_grid: drum_grid_inner,
        drum_knobs: chunks[7],
        drum_fader: drum_fader,
        extra: extra_idx.map(|i| chunks[i]),
        activity_bar: chunks[activity_idx],
    }
}
```

- [ ] **Step 4: Build (expect errors in ui/mod.rs and mouse.rs — fix in next tasks)**

Run: `cargo build 2>&1 | head -80`
Note the errors for fixing in Tasks 8 and 9.

- [ ] **Step 5: Commit**

```bash
git add src/ui/layout.rs
git commit -m "refactor: new ComputedLayout with per-panel rects, dynamic constraints"
```

---

### Task 8: Render Dispatch Update (ui/mod.rs)

**Files:**
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Update render() to use new layout**

Replace the `render()` function (lines 25-60) to use new `compute_layout()` signature and render all panels:

```rust
pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();
    let ly = compute_layout(size, &app.ui.panel_vis, app.ui.show_help);

    transport_bar::render_transport(f, ly.transport, app);

    // Synth A
    if app.ui.panel_vis.synth_a_knobs {
        render_volume_fader(f, ly.synth_a_fader, /* synth A volume */, /* focused */);
        synth_knobs::render_synth_knobs(f, ly.synth_a_knobs, app, SynthId::A);
    } else {
        render_collapsed_panel(f, ly.synth_a_knobs, "SYNTH A KNOBS", &app.ui.panel_vis);
    }
    if app.ui.panel_vis.synth_a_grid {
        synth_grid::render_synth_grid(f, ly.synth_a_grid, app, SynthId::A);
    } else {
        render_collapsed_panel(f, ly.synth_a_grid, "SYNTH A GRID", &app.ui.panel_vis);
    }

    // Synth B (identical structure)
    if app.ui.panel_vis.synth_b_knobs {
        render_volume_fader(f, ly.synth_b_fader, /* synth B volume */, /* focused */);
        synth_knobs::render_synth_knobs(f, ly.synth_b_knobs, app, SynthId::B);
    } else {
        render_collapsed_panel(f, ly.synth_b_knobs, "SYNTH B KNOBS", &app.ui.panel_vis);
    }
    if app.ui.panel_vis.synth_b_grid {
        synth_grid::render_synth_grid(f, ly.synth_b_grid, app, SynthId::B);
    } else {
        render_collapsed_panel(f, ly.synth_b_grid, "SYNTH B GRID", &app.ui.panel_vis);
    }

    render_separator(f, ly.separator);

    // Drums
    if app.ui.panel_vis.drum_grid {
        render_volume_fader(f, ly.drum_fader, /* drum vol */, /* focused */);
        drum_grid::render_drum_grid(f, ly.drum_grid, app);
    } else {
        render_collapsed_panel(f, ly.drum_grid, "DRUM GRID", &app.ui.panel_vis);
    }
    if app.ui.panel_vis.drum_knobs {
        knobs::render_knobs(f, ly.drum_knobs, app);
    } else {
        render_collapsed_panel(f, ly.drum_knobs, "DRUM KNOBS", &app.ui.panel_vis);
    }

    // Extras
    if let Some(extra) = ly.extra {
        if app.ui.show_help {
            help_overlay::render_help(f, extra);
        } else if app.ui.panel_vis.waveform {
            waveform::render_waveform(f, extra, app);
        }
    }
    render_activity_bar(f, ly.activity_bar, app);
}
```

- [ ] **Step 2: Add render_collapsed_panel() function**

```rust
fn render_collapsed_panel(f: &mut Frame, area: Rect, title: &str, _vis: &PanelVisibility) {
    use ratatui::widgets::{Block, Borders};
    let toggle = Span::styled(" [.] ", Style::default().fg(theme::DIM_GRAY));
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM)
        .border_style(Style::default().fg(theme::DIM_GRAY))
        .title(format!(" {} ", title))
        .title_alignment(Alignment::Left);
    // Render [.] at right edge of top border
    f.render_widget(block, area);
    let toggle_x = area.right().saturating_sub(5);
    f.render_widget(toggle, Rect::new(toggle_x, area.y, 5, 1));
}
```

- [ ] **Step 3: Add [X] toggle rendering to expanded panels**

For each expanded panel, render a lit `[X]` at the top-right of its border. Add a helper:

```rust
fn render_panel_toggle(f: &mut Frame, area: Rect, visible: bool) {
    let (label, color) = if visible {
        ("[X]", theme::AMBER)
    } else {
        ("[.]", theme::DIM_GRAY)
    };
    let toggle = Span::styled(format!(" {} ", label), Style::default().fg(color));
    let x = area.right().saturating_sub(5);
    f.render_widget(toggle, Rect::new(x, area.y, 5, 1));
}
```

Call `render_panel_toggle(f, area, true)` after rendering each expanded panel.

- [ ] **Step 4: Remove render_synth_collapsed()**

Delete the old `render_synth_collapsed()` function (lines 107-113) — replaced by `render_collapsed_panel()`.

- [ ] **Step 5: Build**

Run: `cargo build 2>&1 | head -80`
Fix compilation errors. Note: `synth_knobs::render_synth_knobs` and `synth_grid::render_synth_grid` need a `SynthId` parameter — this is addressed in Task 12.

- [ ] **Step 6: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: render dispatch with per-panel collapse and [X] toggles"
```

---

### Task 9: Parameterize Synth Rendering

**Files:**
- Modify: `src/ui/synth_knobs.rs`
- Modify: `src/ui/synth_grid.rs`

- [ ] **Step 1: Update synth_knobs::render_synth_knobs signature**

Change (line ~64):
```rust
pub fn render_synth_knobs(f: &mut Frame, area: Rect, app: &App, synth_id: SynthId)
```

Inside, replace references to `app.synth_pattern.params` with:
```rust
let (pattern, ui) = match synth_id {
    SynthId::A => (&app.synth_a_pattern, &app.ui.synth_a),
    SynthId::B => (&app.synth_b_pattern, &app.ui.synth_b),
};
```

Then use `pattern.params` and `ui.ctrl_field` throughout.

- [ ] **Step 2: Update synth_grid::render_synth_grid signature**

Change (line ~17):
```rust
pub fn render_synth_grid(f: &mut Frame, area: Rect, app: &App, synth_id: SynthId)
```

Inside, use the appropriate pattern/UI state based on synth_id. Update the title to show "SYNTH A" or "SYNTH B".

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles. Both synth sections render correctly.

- [ ] **Step 4: Commit**

```bash
git add src/ui/synth_knobs.rs src/ui/synth_grid.rs
git commit -m "refactor: parameterize synth rendering with SynthId"
```

---

### Task 10: Mouse Hit-Testing Update

**Files:**
- Modify: `src/mouse.rs`

- [ ] **Step 1: Update compute_layout() call**

Replace the `compute_layout()` call (currently `compute_layout(size, app.ui.synth_collapsed, ...)`) with:
```rust
let ly = compute_layout(term_size, &app.ui.panel_vis, app.ui.show_help);
```

- [ ] **Step 2: Add [X] toggle hit-testing**

Add a function to check if a click is on any panel's [X] toggle:

```rust
fn hit_test_panel_toggle(col: u16, row: u16, ly: &ComputedLayout, vis: &PanelVisibility) -> Option<PanelToggle> {
    // Check if col is within last 5 columns of any panel's top border
    let panels = [
        (ly.synth_a_knobs, PanelToggle::SynthAKnobs),
        (ly.synth_a_grid, PanelToggle::SynthAGrid),
        (ly.synth_b_knobs, PanelToggle::SynthBKnobs),
        (ly.synth_b_grid, PanelToggle::SynthBGrid),
        (ly.drum_grid, PanelToggle::DrumGrid),
        (ly.drum_knobs, PanelToggle::DrumKnobs),
    ];
    for (area, toggle) in &panels {
        if row == area.y && col >= area.right().saturating_sub(5) {
            return Some(*toggle);
        }
    }
    None
}
```

Add `PanelToggle` enum in `app.rs` or `mouse.rs`:
```rust
#[derive(Clone, Copy)]
enum PanelToggle {
    SynthAKnobs, SynthAGrid,
    SynthBKnobs, SynthBGrid,
    DrumGrid, DrumKnobs,
}
```

- [ ] **Step 3: Wire toggle clicks to PanelVisibility**

In `handle_left_down()`, check for toggle clicks first (before other hit-testing):

```rust
if let Some(toggle) = hit_test_panel_toggle(col, row, &ly, &app.ui.panel_vis) {
    match toggle {
        PanelToggle::SynthAKnobs => app.ui.panel_vis.synth_a_knobs = !app.ui.panel_vis.synth_a_knobs,
        PanelToggle::SynthAGrid => app.ui.panel_vis.synth_a_grid = !app.ui.panel_vis.synth_a_grid,
        // ... etc for all panels
    }
    return;
}
```

- [ ] **Step 4: Update synth knob hit-testing for dual synths**

The existing `hit_test_synth_knobs()` must be called for both synth areas. In `handle_left_down()`, check which synth area was clicked:

```rust
if panel_vis.synth_a_knobs && ly.synth_a_knobs.contains(Position::new(col, row)) {
    if let Some(field) = hit_test_synth_knobs(col, row, ly.synth_a_knobs) {
        app.ui.focus = FocusSection::SynthAControls;
        handle_synth_knobs_click(app, field, row, SynthId::A);
    }
}
// Same for synth B
```

- [ ] **Step 5: Update remaining hit areas**

Update all mouse hit-testing to use new `ComputedLayout` field names:
- `ly.synth_section` -> `ly.synth_a_knobs` / `ly.synth_a_grid` / `ly.synth_b_knobs` / `ly.synth_b_grid`
- `ly.drum_area` -> `ly.drum_grid`
- `ly.knobs` -> `ly.drum_knobs`

- [ ] **Step 6: Build and test**

Run: `cargo build && cargo test`
Expected: All pass

- [ ] **Step 7: Commit**

```bash
git add src/mouse.rs src/app.rs
git commit -m "feat: mouse hit-testing for [X] toggles and dual synth areas"
```

---

## Chunk 4: Keyboard and Transport

### Task 11: Transport Bar — Three Status Lines

**Files:**
- Modify: `src/ui/transport_bar.rs`

- [ ] **Step 1: Add Synth B status line**

In the transport bar render function (line ~18), add a third status line between the existing synth and drum lines:

```
Line 2: Synth A [pattern/queued] Kit: name  Loop ON  S:length
Line 3: Synth B [pattern/queued] Kit: name  Loop ON  S:length
Line 4: Drum    [pattern/queued] Kit: name  Loop ON  D:length
```

Use `app.ui.synth_a` and `app.ui.synth_b` for the respective pattern/kit/loop data. Use `app.transport.loop_config.synth_a_length` and `synth_b_length`.

- [ ] **Step 2: Verify TRANSPORT_HEIGHT = 7 is sufficient**

The transport block has borders (2 lines) + content (4 lines for line1/2/3/4 + title) = needs height 7. Confirm layout constant is 7 (set in Task 7).

- [ ] **Step 3: Update transport bar mouse hit-testing**

In `src/mouse.rs`, update the transport area click handler to account for the new Synth B row. The transport now has 4 content rows instead of 3. If there's click-to-focus logic for the synth/drum status rows in the transport, update the row offsets:
- Row 0: play/BPM/gauges
- Row 1: Synth A status (click -> focus SynthAControls)
- Row 2: Synth B status (click -> focus SynthBControls) **NEW**
- Row 3: Drum status (click -> focus DrumGrid)

This maintains the project's "layout must match in two places" invariant.

- [ ] **Step 4: Build and verify**

Run: `cargo build`
Expected: Transport bar shows 3 instrument lines.

- [ ] **Step 5: Commit**

```bash
git add src/ui/transport_bar.rs src/mouse.rs
git commit -m "feat: transport bar with Synth A, Synth B, and Drum status lines"
```

---

### Task 12: Focus-Aware Key Bindings

**Files:**
- Modify: `src/keys.rs`

- [ ] **Step 1: Update F2 to bulk synth toggle**

Replace the F2 handler (lines 159-162):

```rust
KeyCode::F(2) => {
    let all_synth_visible = app.ui.panel_vis.synth_a_knobs
        && app.ui.panel_vis.synth_a_grid
        && app.ui.panel_vis.synth_b_knobs
        && app.ui.panel_vis.synth_b_grid;
    let new_state = !all_synth_visible;
    app.ui.panel_vis.synth_a_knobs = new_state;
    app.ui.panel_vis.synth_a_grid = new_state;
    app.ui.panel_vis.synth_b_knobs = new_state;
    app.ui.panel_vis.synth_b_grid = new_state;
    return;
}
```

- [ ] **Step 2: Add helper to determine active synth from focus**

```rust
fn focused_synth(focus: FocusSection) -> Option<SynthId> {
    match focus {
        FocusSection::SynthAGrid | FocusSection::SynthAControls => Some(SynthId::A),
        FocusSection::SynthBGrid | FocusSection::SynthBControls => Some(SynthId::B),
        _ => None,
    }
}
```

- [ ] **Step 3: Update pattern selection keys (Q-P)**

In the pattern key handler (lines ~436-445), use `focused_synth()`:

```rust
if let Some(synth_id) = focused_synth(app.ui.focus) {
    let ui = match synth_id {
        SynthId::A => &mut app.ui.synth_a,
        SynthId::B => &mut app.ui.synth_b,
    };
    if shift {
        // immediate switch
        ui.active_pattern = idx;
        app.send_synth_pattern(synth_id);
    } else {
        // queue
        ui.queued_pattern = Some(idx);
    }
} else {
    // drum pattern selection (existing logic)
}
```

- [ ] **Step 4: Update kit selection keys (1-8)**

Similar focus-aware routing for kit selection.

- [ ] **Step 5: Update loop length key (L / Shift+L)**

Route to `synth_a_length` or `synth_b_length` based on focused synth:

```rust
if let Some(synth_id) = focused_synth(app.ui.focus) {
    let len = match synth_id {
        SynthId::A => &mut app.transport.loop_config.synth_a_length,
        SynthId::B => &mut app.transport.loop_config.synth_b_length,
    };
    *len = match *len { 8 => 16, 16 => 24, 24 => 32, _ => 8 };
} else {
    // drum loop length (existing)
}
```

- [ ] **Step 6: Update synth note trigger keys**

Real-time note triggers (Z-M row) should use the focused synth:
```rust
let synth_id = focused_synth(app.ui.focus).unwrap_or(SynthId::A);
app.tx_to_audio.send(UiToAudio::TriggerSynth(synth_id, note)).ok();
```

- [ ] **Step 7: Build and test**

Run: `cargo build && cargo test`
Expected: All pass

- [ ] **Step 8: Commit**

```bash
git add src/keys.rs
git commit -m "feat: focus-aware key bindings for dual synth, F2 bulk toggle"
```

---

## Chunk 5: PlaybackPosition and App Integration

### Task 13: Wire PlaybackPosition for Dual Synth

**Files:**
- Modify: `src/app.rs` (or `src/main.rs` — wherever PlaybackPosition is consumed)

- [ ] **Step 1: Update PlaybackPosition handler**

Find where `AudioToUi::PlaybackPosition` is matched (likely in the main event loop or `App` method). Update to read dual synth fields:

```rust
AudioToUi::PlaybackPosition {
    global_step, beat, is_bar_start, triggered,
    synth_a_triggered, synth_b_triggered,
    drum_step, synth_a_step, synth_b_step,
} => {
    app.ui.synth_a.playback_step = synth_a_step;
    app.ui.synth_b.playback_step = synth_b_step;
    if synth_a_triggered { app.ui.synth_a.flash = 3; }
    if synth_b_triggered { app.ui.synth_b.flash = 3; }
    // ... existing drum handling
}
```

- [ ] **Step 2: Build and test**

Run: `cargo build && cargo test`

- [ ] **Step 3: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "feat: wire dual synth playback position to UI state"
```

---

### Task 14: App Helper Methods for Dual Synth

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add send_synth_pattern helper**

```rust
impl App {
    pub fn send_synth_pattern(&self, id: SynthId) {
        let pattern = match id {
            SynthId::A => &self.synth_a_pattern,
            SynthId::B => &self.synth_b_pattern,
        };
        self.tx_to_audio.send(UiToAudio::SetSynthPattern(id, pattern.clone())).ok();
    }
}
```

- [ ] **Step 2: Update existing synth helper methods**

Find and update any existing methods like `switch_synth_pattern()`, `switch_synth_kit()`, `queue_synth_pattern()` to accept `SynthId` and route to the appropriate pattern/UI state.

- [ ] **Step 3: Build and test**

Run: `cargo build && cargo test`

- [ ] **Step 4: Commit**

```bash
git add src/app.rs
git commit -m "feat: App helper methods for dual synth pattern/kit management"
```

---

## Chunk 6: Project Storage and Integration

### Task 15: ProjectFile Dual Synth Storage

**Files:**
- Modify: `src/sequencer/project.rs`

- [ ] **Step 1: Add Synth B fields to ProjectFile**

In `ProjectFile` struct (lines 369-400), add:

```rust
#[serde(default)]
pub synth_b_kits: Vec<SynthKitData>,
#[serde(default)]
pub active_synth_b_kit: usize,
#[serde(default)]
pub synth_b_patterns: Vec<SynthPatternData>,
#[serde(default)]
pub active_synth_b_pattern: usize,
```

All use `#[serde(default)]` for backward compatibility.

- [ ] **Step 2: Update save logic**

In the save method, populate synth B fields from `app.synth_b_pattern` and `app.ui.synth_b`.

- [ ] **Step 3: Update load logic**

In the load method, restore synth B state. If synth_b fields are empty/default (old file), Synth B gets default empty patterns and default kit.

- [ ] **Step 4: Write a roundtrip test**

```rust
#[test]
fn test_project_roundtrip_dual_synth() {
    let mut project = ProjectFile::default();
    // Set up synth B data
    project.synth_b_patterns = vec![SynthPatternData { /* ... */ }];
    project.synth_b_kits = vec![SynthKitData { name: "Test".into(), params: SynthParams::default() }];
    project.active_synth_b_pattern = 1;
    project.active_synth_b_kit = 2;

    let json = serde_json::to_string(&project).unwrap();
    let loaded: ProjectFile = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.synth_b_patterns.len(), 1);
    assert_eq!(loaded.active_synth_b_pattern, 1);
    assert_eq!(loaded.active_synth_b_kit, 2);
}
```

- [ ] **Step 5: Test backward compatibility**

```rust
#[test]
fn test_old_project_loads_with_synth_b_defaults() {
    // Minimal valid JSON without any synth_b fields (simulates old project file)
    let json = r#"{
        "textstep": {"format_version": 1, "app_version": "0.1.0"},
        "patterns": [],
        "synth_patterns": [],
        "synth_kits": []
    }"#;
    let loaded: ProjectFile = serde_json::from_str(json).unwrap();
    // Synth B fields should all be at defaults
    assert!(loaded.synth_b_patterns.is_empty());
    assert!(loaded.synth_b_kits.is_empty());
    assert_eq!(loaded.active_synth_b_pattern, 0);
    assert_eq!(loaded.active_synth_b_kit, 0);
    // Synth A fields should also load fine
    assert!(loaded.synth_patterns.is_empty());
}
```

- [ ] **Step 6: Build and test**

Run: `cargo build && cargo test`
Expected: All pass, including new roundtrip tests.

- [ ] **Step 7: Commit**

```bash
git add src/sequencer/project.rs
git commit -m "feat: ProjectFile dual synth storage with backward compatibility"
```

---

### Task 16: Preset Browser Synth B Support

**Files:**
- Modify: `src/presets/mod.rs`
- Modify: `src/presets/synth_presets.rs`
- Modify: `src/presets/synth_pattern_presets.rs`

- [ ] **Step 1: Update preset browser state to track target synth**

In `src/presets/mod.rs`, add a `target_synth: SynthId` field to the preset browser state. Set it from the currently focused synth when the browser is opened.

- [ ] **Step 2: Update synth preset application methods**

The preset browser has entry points for applying:
- Synth sound presets (kit/params) — route to `app.synth_a_pattern.params` or `app.synth_b_pattern.params` based on `target_synth`
- Synth pattern presets (step data) — route to `app.synth_a_pattern.steps` or `app.synth_b_pattern.steps` based on `target_synth`

Find all methods in `presets/mod.rs` that write to `app.synth_pattern` and update them to use `target_synth` for routing. Also update the corresponding `send_synth_pattern()` call to pass the correct `SynthId`.

- [ ] **Step 3: Update preset browser opening in keys.rs**

Where the preset browser is opened (likely a key binding), set `target_synth` from the current focus:
```rust
preset_browser.target_synth = focused_synth(app.ui.focus).unwrap_or(SynthId::A);
```

- [ ] **Step 4: Build and test**

Run: `cargo build && cargo test`

- [ ] **Step 5: Commit**

```bash
git add src/presets/
git commit -m "feat: preset browser loads into focused synth (A or B)"
```

---

### Task 17: Integration Testing and Polish

**Files:**
- All modified files

- [ ] **Step 1: Full build and test suite**

Run: `cargo build && cargo test`
Expected: All pass

- [ ] **Step 2: Manual testing checklist**

Run the app with `cargo run` and verify:

1. [ ] Transport bar shows 3 lines: Synth A, Synth B, Drum
2. [ ] Synth A section renders with knobs + grid (expanded by default)
3. [ ] Synth B section renders collapsed by default (2-line bars)
4. [ ] Click [.] on Synth B knobs -> expands, shows [X]
5. [ ] Click [X] on Synth A knobs -> collapses, drum grid expands
6. [ ] F2 toggles all synth panels at once
7. [ ] Tab cycles through: Transport -> SA Controls -> SA Grid -> SB Controls -> SB Grid -> Drum Grid -> Drum Knobs
8. [ ] Tab skips collapsed panels
9. [ ] Pattern keys (Q-P) apply to focused synth
10. [ ] Kit keys (1-8) apply to focused synth
11. [ ] Loop length (Shift+L) applies to focused synth
12. [ ] Synth A plays notes independently
13. [ ] Synth B plays notes independently when pattern is programmed
14. [ ] Both synths mix correctly to output
15. [ ] Save project, reload -> both synths restored
16. [ ] Load old project file -> Synth B is empty/default, no crash

- [ ] **Step 3: Fix any issues found in manual testing**

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: panel minimize system and dual synth (Synth A / Synth B)"
```

---

## Parallelization Guide

For subagent-driven development, these chunks can be parallelized:

```
Chunk 1 (Data Model) ──────────────────────────┐
                                                ├── Chunk 4 (Keyboard/Transport)
Chunk 2 (Audio Engine) ── depends on Chunk 1 ──┤
                                                ├── Chunk 5 (Playback/App)
Chunk 3 (Layout) ──── depends on Chunk 1 ──────┤
                                                └── Chunk 6 (Project/Integration)
```

- **Chunk 1** must complete first (all other chunks reference its types)
- **Chunks 2 and 3** can run in parallel (audio vs UI, no shared changes)
- **Chunks 4 and 5** can run in parallel after Chunks 2+3
- **Chunk 6** runs last (integration depends on everything)
