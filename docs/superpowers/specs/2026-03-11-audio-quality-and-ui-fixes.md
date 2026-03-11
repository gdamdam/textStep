# Audio Quality & UI Fixes Plan

## Issue 1: Drum Fader Click Offsets [BUG - Quick Fix]

**Root cause:** `mouse.rs:643-653` hit-test logic doesn't match `knobs.rs:54` rendering.
- Rendering: `col_width = inner.width / 11` (all KNOB_FIELDS)
- Hit-test: reserves 8px for buttons, divides remaining by 10, caps index at 9
- Result: Volume, SendReverb, SendDelay sliders are unclickable

**Fix:**
- `src/mouse.rs` `hit_test_knobs_panel()`: remove button_width reservation, use `col_width = inner_w / 11`, remove `.min(9)` cap, use `KNOB_FIELDS.len()` instead of hardcoded 10

**Files:** `src/mouse.rs`

---

## Issue 2: Delay & Reverb Not Audible [Quick Fix + Tuning]

**Root causes:**
1. All send_reverb and send_delay defaults are 0.0 - no signal reaches effects
2. Reverb wet = amount * 0.4 (max 40%) - too quiet
3. Delay wet = 0.3 + feedback * 0.2 (max 50%) - too quiet

**Fix - Phase A (defaults & gain staging):**
- `src/sequencer/drum_pattern.rs`: change DrumTrackParams defaults: `send_reverb: 0.15, send_delay: 0.0` (subtle default reverb)
- `src/sequencer/synth_pattern.rs`: change SynthParams defaults: `send_reverb: 0.2, send_delay: 0.0`
- `src/audio/effects.rs` Reverb: change wet to `amount * 0.7` (was 0.4)
- `src/audio/effects.rs` Delay: change wet to `0.4 + feedback * 0.4` (was 0.3 + fb * 0.2)
- Update existing presets that explicitly set send values

**Fix - Phase B (more musical delay subdivisions):**
- Current: 4 divisions (8th, dotted 8th, quarter, triplet)
- Add: 1/16, 1/16 dotted, 1/8 triplet, 1/4 dotted, 1/2, 1/2 dotted, whole note
- Expand DelaySub enum and subdivision selection in effects.rs
- Update UI label display in synth_knobs.rs

**Files:** `src/audio/effects.rs`, `src/sequencer/drum_pattern.rs`, `src/sequencer/synth_pattern.rs`, `src/presets/drum_presets.rs`, `src/presets/synth_presets.rs`

---

## Issue 3: LFO Range Too Narrow [Medium Effort]

**Current state:** 10 tempo-synced divisions (1/16 to 32bar), 3 waveforms, global LFO

**Fix - Phase A (expand divisions):**
- Add dotted values: 1/16D, 1/8D, 1/4D, 1/2D
- Add triplet values: 1/16T, 1/8T, 1/4T, 1/2T
- Expand LFO_DIVISIONS array from 10 to ~18 entries
- Update division label display

**Fix - Phase B (add waveforms):**
- Add Sine (currently missing - most basic shape!)
- Add Saw Up, Saw Down
- Expand from 3 to 6 waveforms
- Update waveform label display

**Fix - Phase C (optional, per-voice retrigger):**
- Move LFO from global (engine.rs) to per-voice (synth_voice.rs)
- Add retrigger on note-on
- This is a bigger architectural change, can defer

**Files:** `src/sequencer/synth_pattern.rs` (LFO_DIVISIONS), `src/audio/engine.rs` (Lfo struct, waveform shapes), `src/ui/synth_knobs.rs` (labels)

---

## Issue 4: Filter Doesn't Close [Investigation Needed]

**Current implementation is actually correct:**
- Cytomic SVF, exponential mapping: 20Hz at 0.0, 20kHz at 1.0
- 24dB/oct slope should close effectively

**Possible causes to investigate:**
1. Filter envelope (`filter_env_amount`) may be re-opening the filter even when cutoff is at 0
2. Oscillator signal may be leaking past the filter (check signal path ordering)
3. Sub-oscillator or second oscillator may bypass the filter
4. The perception may be about drum voices which use different inline filters

**Proposed approach:**
- First: add debug logging / listen test to confirm filter behavior at cutoff=0
- Check if filter_env_amount needs to be zeroed for the filter to fully close
- Check signal routing: does any signal bypass Filter24dB?

**Optional refactor:** Extract Filter24dB + Svf to `src/audio/filter.rs` shared module, reuse in drum voices. This improves code quality but is not strictly needed for the bug.

**Files:** `src/audio/synth_voice.rs` (signal path audit), potentially new `src/audio/filter.rs`

---

## Implementation Order

1. **Drum fader clicks** - smallest fix, immediate UX improvement
2. **Delay/reverb gain staging** - high impact, moderate effort
3. **LFO divisions + waveforms** - medium effort, big expressiveness gain
4. **Delay subdivisions** - small addition, musical value
5. **Filter investigation** - needs hands-on testing
6. **Filter refactor to shared module** - optional cleanup
