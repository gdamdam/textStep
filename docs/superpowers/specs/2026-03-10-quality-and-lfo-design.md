# TextStep: Synth LFO + Audio Quality + Code Quality Design

Date: 2026-03-10

## 1. Synth LFO

### Data Model

New fields in `SynthParams` (`synth_pattern.rs`):
- `lfo_waveform: u8` — 0=Exponential, 1=Triangle, 2=Square
- `lfo_division: f32` — normalized 0.0-1.0, mapped to beat divisions: 1/16, 1/8, 1/4, 1/2, 1, 2, 4, 8, 16, 32
- `lfo_depth: f32` — 0.0-1.0
- `lfo_dest: u8` — index into assignable SynthControlField variants (continuous params only)

New `SynthControlField` variants:
- `LfoWaveform` — enum-style, cycles Exp/Tri/Sqr
- `LfoDivision` — enum-style, cycles beat divisions
- `LfoDepth` — continuous slider
- `LfoDest` — enum-style, cycles through assignable params

### DSP

Global `Lfo` struct on `AudioEngine`:
- `phase: f64` — accumulated, resets on transport start
- `tick(sample_rate, bpm, division) -> f32` — returns -1.0 to 1.0
- Phase increment: `(bpm / 60.0) * division_multiplier / sample_rate`
- Waveforms: exponential (exp curve per cycle), triangle, square

Applied in `engine.rs` per-sample: compute `lfo_value`, create local copy of `SynthParams` with modulated destination, pass to `synth_voice.tick()`.

### UI

New row group 3 (LFO), AMP becomes row group 4. `SYNTH_KNOBS_HEIGHT` 27->30, `SYNTH_SECTION_HEIGHT` 33->36.

```
 LFO ──────────────────────────────────
  [Exp]   1/4    ████░░░░  [FilterCutoff]
  Wave    Div     Depth     Dest
```

### Navigation

Added to `SYNTH_CTRL_ROWS` as row 3. Left/right across 4 LFO fields. AMP shifts to row 4.

## 2. Sound Engineer Fixes

### 2a. PolyBLEP Anti-Aliasing

Add `poly_blep(t, dt)` correction to saw and square in `Oscillator::tick()` (`synth_voice.rs`). ~4 arithmetic ops per sample.

### 2b. Summing Bus Headroom

`engine.rs`: multiply mixed signal by 0.5 before compressor to prevent double-saturation.

### 2c. Scale Reverb Lengths by Sample Rate

`effects.rs`: `Reverb::new(sample_rate)` scales comb/allpass lengths by `sample_rate / 44100.0`.

### 2d. Fix Hardcoded 48kHz in TubeSaturator

Add `sample_rate` field to `TubeSaturator`. Replace `1.0 / 48000.0` with `1.0 / self.sample_rate` in `set_drive()`.

### 2e. Buffer-Independent Peak Meter Decay

`engine.rs`: replace fixed `0.85` coefficient with `(-buffer_size / (0.06 * sample_rate)).exp()`.

### 2f. Stereo Panning Per Drum Track

Add `pan: f32` to `DrumTrackParams` (default 0.5 = center). Apply equal-power pan law in engine mixing. Output becomes true stereo. Follows "Adding a Drum Parameter" pattern from CLAUDE.md.

## 3. Rust Code Quality

### 3a. DrumPattern and SynthPattern Copy

Add `Copy` to derive macros. All fields are Copy-compatible.

### 3b. Result from start_audio_stream

Replace 4 `expect()` calls with `Result<Stream, String>`. Display error gracefully in main.

### 3c. DrumTrackId in TriggerDrum

Change `TriggerDrum(usize)` to `TriggerDrum(DrumTrackId)`. Update keys.rs, mouse.rs, engine.rs.

### 3d. Pre-allocate Stereo Fallback Buffer

Add `fallback_buf: Vec<f32>` to `AudioEngine`. Zero and reuse instead of allocating per callback.

## 4. Computed Layout Struct

New `ComputedLayout` struct in `ui/layout.rs` with all rects (transport, drum_grid, synth sub-areas, etc.). Single `compute_layout(frame, synth_expanded) -> ComputedLayout` function. Both `ui/mod.rs` and `mouse.rs` consume it. Eliminates layout duplication bug class.
