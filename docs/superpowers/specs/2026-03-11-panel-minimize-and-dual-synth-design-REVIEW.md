# Spec Review: Panel Minimize System & Dual Synth Design

Reviewer: Code Review Agent
Date: 2026-03-11
Spec: `docs/superpowers/specs/2026-03-11-panel-minimize-and-dual-synth-design.md`

---

## Summary

The spec proposes two coupled features: per-panel minimize toggles and a second independent synth. The design is well-structured and the dependency relationship (minimize enables dual synth) is sound. However, there are several gaps that would block or confuse implementation.

---

## Critical Issues (Must Fix)

### C1. `compute_layout()` redesign is unspecified

The existing `compute_layout()` in `layout.rs` is the **single source of truth** consumed by both `ui/mod.rs` and `mouse.rs`. It currently takes `synth_collapsed: bool` and returns a `ComputedLayout` with fixed fields (`synth_section`, `synth_knobs`, `synth_grid`, etc.).

The spec says panels switch from `Constraint::Length(N)` to `Constraint::Length(1)` but does not specify:
- The new signature for `compute_layout()` (it must accept `PanelVisibility`)
- The new `ComputedLayout` struct fields (needs `synth_a_knobs`, `synth_a_grid`, `synth_b_knobs`, `synth_b_grid`, etc.)
- How `Constraint::Min()` is distributed -- which panel(s) expand? Only the drum grid? All visible panels proportionally?

This is the most architecturally load-bearing change and needs explicit design.

### C2. Audio engine duplication strategy is underspecified

The current `AudioEngine` has a single `synth_pattern: SynthPattern`, `synth_voice: SynthVoice`, `lfo: Lfo`, `synth_gate_samples: u32`, and `synth_note_end_step: Option<usize>`. The spec says "two SynthVoice instances, two independent LFO structs" but does not address:
- The `synth_gate_samples` and `synth_note_end_step` per-voice tracking
- Whether to extract a `SynthEngine` sub-struct (recommended) or inline duplicate all fields
- The `synth_saturator: TubeSaturator` -- is that per-synth or shared?
- How `AudioToUi::PlaybackPosition` reports dual synth state (currently has single `synth_triggered: bool` and `synth_step: usize`)

### C3. Collapsed panel height contradiction

Line 39 says "Height: 1 line" and line 178 adds `COLLAPSED_PANEL_HEIGHT = 1`. But the ASCII art on lines 34-37 shows TWO lines (a top border and a bottom border). A 1-line collapsed panel that includes a visible border character on a single row is feasible, but the spec should clarify: is it 1 row total (single horizontal line with embedded label), or 2 rows (top border + bottom border)?

The existing `SYNTH_COLLAPSED_HEIGHT` is 3 (border + step row + border). Jumping to 1 may look cramped. Recommend clarifying the visual spec with an exact mockup.

---

## Important Issues (Should Fix)

### I1. `AudioToUi::PlaybackPosition` needs dual synth fields

The current message has `synth_triggered: bool` and `synth_step: usize`. With two synths, this needs `synth_a_triggered`, `synth_b_triggered`, `synth_a_step`, `synth_b_step`. The spec mentions messages for UI-to-Audio but omits Audio-to-UI changes entirely.

### I2. Transport bar layout change (6 -> 7 lines) is underspecified

The spec adds a Synth B status line to the transport bar. But `transport_bar.rs` currently renders a tightly packed 4-content-line layout. The spec should clarify:
- Which existing line gets the Synth B info added, or where the new line goes
- Whether the transport bar mockup (lines 132-136) is authoritative -- it shows 4 content lines which, with top+bottom borders, is 6 lines, not 7

### I3. FocusSection::next()/prev() with dynamic skip logic is unspecified

The spec says "collapsed panels are skipped in the Tab cycle." Currently `FocusSection::next()` and `prev()` are pure state-machine methods with no external input. The new design needs access to `PanelVisibility` to skip collapsed sections. The spec should specify whether:
- `next()`/`prev()` take `PanelVisibility` as a parameter
- Or a wrapper function handles the skip logic
- What happens when ALL panels are collapsed (infinite loop guard)

### I4. Separator line fate is unclear

The current layout has a `SEPARATOR_HEIGHT = 1` line between synth and drums. With 6+ panels stacked vertically, the spec does not say whether separators appear between every panel, only between synth/drum sections, or are eliminated entirely in favor of the panel title bars serving as visual separators.

### I5. Three independent FX chains -- CPU and memory concern

The spec proposes three full reverb + delay chains. The current `ReverbEffect` is likely non-trivial (reverb algorithms need large delay buffers). Tripling this may be a concern on resource-constrained systems. The spec should acknowledge this and suggest whether to use lighter algorithms or share FX with send routing.

### I6. No specification for waveform/spectrum display with dual synth

The waveform panel currently shows a single mixed output or single synth signal. The spec lists the waveform panel as collapsible but does not specify what it displays with two synths active. Show combined? Show focused synth only?

---

## Suggestions (Nice to Have)

### S1. Consider a `SynthInstance` struct to reduce field duplication

Rather than duplicating 6+ fields per synth in `UiState` (`synth_X_active_pattern`, `synth_X_queued_pattern`, etc.), define:

```rust
struct SynthInstanceState {
    active_pattern: usize,
    queued_pattern: Option<usize>,
    active_kit: usize,
    cursor_step: usize,
    ctrl_field: SynthControlField,
    playback_step: usize,
    flash: u8,
}
```

Then `UiState` holds `synth_a: SynthInstanceState, synth_b: SynthInstanceState`. This reduces duplication in state, serialization, and every function that operates on synth state. Same pattern applies to the audio engine side.

### S2. Consider keyboard shortcut for panel toggles

The spec says "no dedicated shortcut initially (mouse-first)." Given that this is a TUI app where keyboard-first workflows are common, consider at least reserving key bindings (e.g., F2-F8 for each panel) even if not implemented in v1.

### S3. Preset browser context

The spec says "each synth independently browses and loads presets" but does not specify how the preset browser knows which synth to apply to. Presumably it uses the current `FocusSection`, but this should be stated explicitly.

### S4. Demo project migration

The `demo_project()` function in `project.rs` creates default content. Should it populate Synth B with something interesting, or leave it empty? Worth specifying for first-run experience.

---

## What the Spec Does Well

- Clear dependency ordering: minimize system is prerequisite for dual synth
- Backward-compatible migration strategy using `#[serde(default)]`
- Explicit decision to use parameterized messages (`SetSynthPattern(SynthId, ...)`) over duplicated variants
- Testing strategy covers the key risk areas (panel visibility state, layout computation, save/load roundtrip)
- Correctly identifies that F2 collapse toggle becomes redundant

---

## Recommendation

**Do not begin implementation until C1-C3 are resolved.** The `compute_layout()` redesign (C1) and audio engine structure (C2) are the two highest-risk areas and need concrete design before coding starts. The remaining Important issues can be resolved during implementation but should be documented as known gaps.
