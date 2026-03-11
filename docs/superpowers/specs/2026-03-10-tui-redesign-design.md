# TextStep TUI Redesign — Design Spec

> Date: 2026-03-10

## Overview

Redesign the TextStep TUI to be more modern, sleek, intuitive, and easy to understand while maintaining a 90s hardware vibe with ANSI/ASCII decorations.

## Aesthetic Direction

**Hardware / Brushed Metal** base with **Synthwave Neon** accents.

Think: Akai MPC / E-mu SP-1200 hardware aesthetic meets outrun neon.

### Color Palette

| Role | Color | RGB | Usage |
|------|-------|-----|-------|
| Background | Dark gray | `#1a1a1a` | Main background |
| Surface | Lighter gray | `#2a2a2a` | Raised surfaces, inactive fills |
| Borders (unfocused) | Mid gray | `#3a3a3a` | Section borders when not focused |
| Dim text | Gray | `#888888` | Secondary labels, inactive items |
| Primary text | Light gray | `#d4d4d4` | Main text content |
| **Amber** | Warm amber | `#e8a838` | Active steps, gauge fills, primary data |
| **Cyan** | Electric cyan | `#61dafb` | Transport state (play/pause), beat LEDs, playhead, cursor, focused borders |
| **Hot Pink** | Neon pink | `#ff6b9d` | Focus/selection, current track highlight, record indicator |
| **Gold** | Bright gold | `#ffd700` | Queued patterns, warnings |
| Mute | Red | `Color::Red` | [M] indicator when active |
| Solo | Green | `Color::Green` | [S] indicator when active |

### Box Drawing

Heavy-weight box drawing characters throughout:
- Corners: `┏ ┓ ┗ ┛`
- Sides: `┃` (vertical), `━` (horizontal)
- Junctions: `┣ ┫ ┻ ┳ ╋`
- Section dividers: `╶── LABEL ──╴`

### Step Grid Characters

| State | Downbeat (1,5,9,13) | Off-beat |
|-------|---------------------|----------|
| Active | `■` bright amber | `■` amber |
| Inactive | `□` dim amber outline | `□` dark gray |

Spaced format with gaps: `■ □ □ □ ■ □ □ □`

Bar separator: `┃` between steps 16 and 17.

## Layout Structure

### Vertical Stack (top to bottom)

```
┏━━ Transport Bar ━━━━━━━━━━━━━━━━━━━━━━━━┓  4 rows
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
┏━ SYNTH [F2 collapse] ━━━━━━━━━━━━━━━━━━━┓  ~28 rows (expanded)
┃  OSC1/OSC2 + ENV1/ENV2/FILT + AMP       ┃   3 rows (collapsed: step row only)
┃  Step grid                               ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
┏━ DRUM MACHINE ┃ P1: Name ━━━━━━━━━━━━━━━┓  Min 22 rows
┃  8-track step grid                       ┃
┃  Vertical slider panel                   ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
┏━ Spectrum (toggleable) ━━━━━━━━━━━━━━━━━━┓  11 rows (when visible)
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
 [Kick] [Snare] ... activity bar              1 row
```

### Synth Section — Collapsible (F2)

**Expanded**: full params + ADSR curves + step grid
**Collapsed**: step row only (single line showing pattern)

When collapsed, the drum machine section gets the freed vertical space.

### Minimum Terminal Size

Target: ~100 columns wide, ~40 rows tall (expanded synth).

## Component Designs

### Transport Bar (4 rows)

```
┏━━ TextStep - Demo Beats* ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃  ▶ PLAY   BPM: 125.0   ●○○○   Swing: 50%                                     ● REC  ┃
┃  Synth  Pattern: [q] w e r t y u i o p   Kit: [1] 2 3 4 5 6 7 8   Loop [ON] S:32    ┃
┃  Drum   Pattern: [q] w e r t y u i o p   Kit: [1] 2 3 4 5 6 7 8   Loop [ON] D:16    ┃
┃                                              VOL ████░░   CMP ██░░   SAT ░░░░        ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

- **Line 1**: Play state (cyan) + BPM + Beat LEDs + Swing + Record (pink)
- **Line 2**: Synth pattern/kit selectors + synth loop indicator
- **Line 3**: Drum pattern/kit selectors + drum loop indicator
- **Line 4**: Master gauges (VOL, CMP, SAT)
- Active pattern/kit: amber background `[q]`
- Queued pattern: gold background
- **All elements clickable**: play/pause toggles on click, BPM scrollable, selectors clickable

### Synth Section (Expanded)

Organized in labeled groups with `╶── LABEL ──╴` dividers:

**OSC1 + OSC2** (top row):
- Wave selector: `[Sqr] Saw Sin Rnd` — click or scroll to cycle
- Vertical sliders: Tune, PWM, Level (OSC1) + Tune, PWM, Level, Detune, Sub (OSC2)

**ENV1 + ENV2 + FILT** (middle row):
- ENV1: ADSR curve visualization (4 rows) with values below
- ENV2: ADSR curve visualization (4 rows) with values below
- FILT: Type selector `[LP] HP BP` + Freq/Res/EnvAmt sliders + own ADSR curve

**AMP** (bottom row):
- Vol, Reverb, Delay, Sat as vertical sliders

**Step grid** (bottom):
- Same format as drum grid: `1 · · · 2 · · · 3 · · · 4 · · ·`
- Bar separator `┃` between bars
- One row labeled "Synth"

### ADSR Curve Visualization

```
   ╱╲
  ╱  ╲____
 ╱        ╲
╱          ╲
.01 .30 .69 .20
A   D   S   R
```

- Curve reshapes dynamically based on actual A/D/S/R values
- Used for ENV1, ENV2, and Filter envelope
- Values shown below, labels at bottom
- Curve drawn in dim gray, values in amber

### Drum Machine Section

**Title bar**: `DRUM MACHINE ┃ P1: House` — pattern number and name

**Step grid**:
- 8 tracks: Kick, Snare, CHH, OHH, Ride, Clap, Cowbell, Tom
- 32 steps in spaced square format with bar separator
- Header row: `1 · · · 2 · · · 3 · · · 4 · · ·`
- Selected track: `>CHH` with pink highlight
- Breathing room: empty rows above/below grid

**Vertical slider panel** (below grid):
- Shows params for **selected track**
- Full-word labels: Tune, Sweep, Color, Snap, Filter, Drive, Decay, Volume, RevSnd, DlySnd
- 5 rows tall per slider
- [M] and [S] buttons to the right
- Mouse wheel adjusts value (0.01 per tick)
- Keyboard: Shift+Up/Down adjusts value (finer: 0.02 increments)
- Values shown below each slider: `.30`

### Activity Bar (1 row, bottom)

```
[Kick] [Snare] [CHH] [OHH] [Ride] [Clap] [Cowbell] [Tom]   CHH Tune: .30   ? Help
```

- Pad indicators flash white on trigger
- Current param tweak displayed when in knobs focus
- Status messages (yellow, timed)
- `? Help` hint

### VU/Scope Panel (toggleable)

Keep current implementation — spectrum analyzer + VU meter. Toggled with existing key.

### Help Overlay

Keep current implementation — 3-column key binding popup.

### Splash Screen

Keep current ASCII logo + matrix rain animation.

## Interaction Improvements

### Click-to-Focus

Clicking anywhere within a bordered section gives that section focus:
- Click in transport box → focus transport
- Click in synth box → focus synth
- Click in drum box → focus drum grid/knobs
- Visual feedback: focused section border changes to cyan

### Clickable Transport

- Click `▶ PLAY` / `■ STOP` → toggle play/pause
- Click `● REC` → toggle record
- Click pattern letters → queue pattern
- Click kit numbers → switch kit
- Click `Loop` → toggle loop
- Scroll wheel on BPM → adjust ±1

### Mouse Wheel on Parameters

- Scroll over any vertical slider → adjust by 0.01 per tick
- Scroll over gauge (CMP, VOL, SAT) → adjust by 0.01 per tick
- Scroll over wave/type selector → cycle options

### Finer Keyboard Increments

- Current: 0.05 per step
- New: 0.02 per step (Shift+Up/Down)
- Alt+Up/Down: 0.02 + audition (play sound)

## Bug Fixes (included in redesign)

### Mouse Click Offset on Resize

**Root cause**: `SYNTH_GRID_HEIGHT` is `3` in `mouse.rs` but `4` in `ui/mod.rs`. The mouse hit-test layout computation doesn't match the render layout.

**Fix**: Use shared constants from a single source. Define all layout heights in one place (e.g., `ui/mod.rs` or a `ui/layout.rs`) and import in both `ui/mod.rs` and `mouse.rs`.

### Layout Constants (single source of truth)

```rust
// ui/layout.rs (new file)
pub const TRANSPORT_HEIGHT: u16 = 5;
pub const SYNTH_KNOBS_HEIGHT: u16 = 8;
pub const SYNTH_GRID_HEIGHT: u16 = 4;
pub const SYNTH_SECTION_HEIGHT: u16 = SYNTH_KNOBS_HEIGHT + SYNTH_GRID_HEIGHT;
pub const SYNTH_COLLAPSED_HEIGHT: u16 = 3;
pub const DRUM_KNOBS_HEIGHT: u16 = 4;
pub const WAVEFORM_HEIGHT: u16 = 11;
pub const ACTIVITY_BAR_HEIGHT: u16 = 1;
```

## Non-Goals (out of scope)

- No changes to audio engine or DSP
- No changes to sequencer logic or project file format
- No changes to key bindings (only adding click targets)
- Help overlay and splash screen kept as-is
