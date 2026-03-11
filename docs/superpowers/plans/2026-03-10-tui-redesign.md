# TUI Redesign Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the TextStep TUI with a hardware/synthwave aesthetic, vertical sliders, ADSR curves, collapsible synth, and improved mouse interaction.

**Architecture:** Pure UI-layer changes. No audio engine, sequencer logic, or file format changes. New `ui/layout.rs` centralizes layout constants. Theme colors replaced in `ui/theme.rs`. Rendering functions rewritten in existing UI files. Mouse handling updated in `mouse.rs` for new layout and scroll wheel.

**Tech Stack:** Rust, ratatui 0.29, crossterm 0.28

**Spec:** `docs/superpowers/specs/2026-03-10-tui-redesign-design.md`

---

## Chunk 1: Foundation — Layout Constants, Theme, Bug Fix

### Task 1: Create shared layout constants

**Files:**
- Create: `src/ui/layout.rs`
- Modify: `src/ui/mod.rs` (remove duplicate constants, add `pub mod layout;`)
- Modify: `src/mouse.rs` (remove local constants, import from layout)

- [ ] **Step 1: Create `src/ui/layout.rs`**

```rust
// src/ui/layout.rs — Single source of truth for all layout dimensions

/// Transport bar: title + status + synth row + drum row + gauges
pub const TRANSPORT_HEIGHT: u16 = 6;

/// Synth section when expanded (params + step grid)
pub const SYNTH_EXPANDED_HEIGHT: u16 = 30;

/// Synth section when collapsed (border + step row + border)
pub const SYNTH_COLLAPSED_HEIGHT: u16 = 3;

/// Minimum drum machine section height (header + 8 tracks + padding + sliders)
pub const DRUM_MIN_HEIGHT: u16 = 22;

/// Drum vertical slider panel height (labels + 5 bars + values + padding)
pub const DRUM_SLIDERS_HEIGHT: u16 = 9;

/// Waveform/spectrum panel height (including border)
pub const WAVEFORM_HEIGHT: u16 = 11;

/// Activity bar (bottom status line)
pub const ACTIVITY_BAR_HEIGHT: u16 = 1;

/// Volume fader column width
pub const FADER_WIDTH: u16 = 3;
```

- [ ] **Step 2: Add `pub mod layout;` to `src/ui/mod.rs`**

Add after line 9 (after `pub mod waveform;`):
```rust
pub mod layout;
```

Remove these local constants from `src/ui/mod.rs` (lines ~21-37):
- `KNOBS_HEIGHT`
- `SYNTH_KNOBS_HEIGHT`
- `SYNTH_GRID_HEIGHT`
- `SYNTH_SECTION_HEIGHT`
- `FADER_WIDTH`
- `WAVEFORM_HEIGHT`

Replace all usages in `src/ui/mod.rs` with `layout::*` imports.

- [ ] **Step 3: Fix mouse.rs layout bug**

In `src/mouse.rs`, find the `layout_chunks()` function. It has a local `SYNTH_SECTION_HEIGHT` computed as `8 + 3 = 11` (should be 12). Replace all hardcoded layout constants with imports from `ui::layout`.

Find: any local `const SYNTH_SECTION_HEIGHT`, `const KNOBS_HEIGHT`, etc.
Replace with: `use crate::ui::layout::*;`

- [ ] **Step 4: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles with no errors

- [ ] **Step 5: Run tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all 23 tests pass

- [ ] **Step 6: Commit**

```bash
git add src/ui/layout.rs src/ui/mod.rs src/mouse.rs
git commit -m "refactor: extract layout constants to ui/layout.rs, fix mouse offset bug"
```

---

### Task 2: Replace theme color palette

**Files:**
- Modify: `src/ui/theme.rs`

- [ ] **Step 1: Rewrite `src/ui/theme.rs` with new palette**

Replace the entire file with:

```rust
// Color constants and style helpers — Hardware/Synthwave theme

use ratatui::style::{Color, Modifier, Style};

// ── Base palette ──────────────────────────────────────────────
pub const BG:          Color = Color::Rgb(26, 26, 26);    // #1a1a1a
pub const SURFACE:     Color = Color::Rgb(42, 42, 42);    // #2a2a2a
pub const BORDER:      Color = Color::Rgb(58, 58, 58);    // #3a3a3a
pub const DIM_TEXT:    Color = Color::Rgb(136, 136, 136);  // #888888
pub const TEXT:        Color = Color::Rgb(212, 212, 212);  // #d4d4d4

// ── Accent colors ─────────────────────────────────────────────
pub const AMBER:       Color = Color::Rgb(232, 168, 56);   // #e8a838
pub const AMBER_BRIGHT:Color = Color::Rgb(255, 200, 80);   // brighter for downbeats
pub const AMBER_DIM:   Color = Color::Rgb(100, 72, 24);    // dim amber outline
pub const CYAN:        Color = Color::Rgb(97, 218, 251);   // #61dafb
pub const PINK:        Color = Color::Rgb(255, 107, 157);  // #ff6b9d
pub const GOLD:        Color = Color::Rgb(255, 215, 0);    // #ffd700

// ── Semantic aliases ──────────────────────────────────────────
pub const ACTIVE_STEP:     Color = AMBER;
pub const ACTIVE_STEP_DOWNBEAT: Color = AMBER_BRIGHT;
pub const INACTIVE_STEP:   Color = BORDER;
pub const INACTIVE_STEP_DOWNBEAT: Color = AMBER_DIM;
pub const PLAYHEAD_BG:     Color = AMBER;
pub const PLAYHEAD_FG:     Color = Color::Rgb(26, 26, 26);
pub const CURSOR_BG:       Color = CYAN;
pub const CURSOR_FG:       Color = Color::Rgb(26, 26, 26);

pub const SELECTED_TRACK:  Color = PINK;
pub const RECORD_COLOR:    Color = PINK;
pub const TRANSPORT_STATE:  Color = CYAN;

pub const MUTED_COLOR:     Color = Color::Red;
pub const SOLOED_COLOR:    Color = Color::Green;

pub const BEAT_LED_ON:     Color = CYAN;
pub const BEAT_LED_OFF:    Color = BORDER;
pub const BAR_START_LED:   Color = Color::Red;
pub const BEAT_BG:         Color = Color::Rgb(30, 30, 40);

pub const FOCUS_BORDER:    Color = CYAN;
pub const NORMAL_BORDER:   Color = BORDER;
pub const TITLE_COLOR:     Color = TEXT;

pub const PARAM_HIGHLIGHT_BG: Color = PINK;
pub const PARAM_HIGHLIGHT_FG: Color = Color::Rgb(26, 26, 26);

pub const QUEUED_BG:       Color = GOLD;
pub const QUEUED_FG:       Color = Color::Rgb(26, 26, 26);

// ── Gauge characters ──────────────────────────────────────────
pub const GAUGE_FILLED: &str = "\u{2588}"; // █
pub const GAUGE_EMPTY:  &str = "\u{2591}"; // ░

// ── Step characters ───────────────────────────────────────────
pub const STEP_ACTIVE:   &str = "\u{25A0}"; // ■
pub const STEP_INACTIVE: &str = "\u{25A1}"; // □

// ── Heavy-weight box drawing ──────────────────────────────────
pub const BOX_TL: &str = "\u{250F}"; // ┏
pub const BOX_TR: &str = "\u{2513}"; // ┓
pub const BOX_BL: &str = "\u{2517}"; // ┗
pub const BOX_BR: &str = "\u{251B}"; // ┛
pub const BOX_H:  &str = "\u{2501}"; // ━
pub const BOX_V:  &str = "\u{2503}"; // ┃
pub const BOX_LT: &str = "\u{2523}"; // ┣
pub const BOX_RT: &str = "\u{252B}"; // ┫

/// Style for a focused section border (heavy-weight).
pub fn focus_border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(FOCUS_BORDER)
    } else {
        Style::default().fg(NORMAL_BORDER)
    }
}

/// Style for a highlighted/selected parameter field.
pub fn param_highlight_style() -> Style {
    Style::default()
        .bg(PARAM_HIGHLIGHT_BG)
        .fg(PARAM_HIGHLIGHT_FG)
        .add_modifier(Modifier::BOLD)
}

/// Renders a horizontal gauge bar like "████░░░░░░".
pub fn gauge_string(value: f32, width: usize) -> String {
    let v = value.clamp(0.0, 1.0);
    let filled = (v * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let mut s = String::with_capacity(width * 3);
    for _ in 0..filled { s.push_str(GAUGE_FILLED); }
    for _ in 0..empty { s.push_str(GAUGE_EMPTY); }
    s
}

/// Renders a percentage string like "80%".
pub fn percent_string(value: f32) -> String {
    let pct = (value.clamp(0.0, 1.0) * 100.0).round() as u32;
    format!("{}%", pct)
}
```

- [ ] **Step 2: Update all files that import old theme constants**

Search for usages of removed constants: `ACTIVE_STEP` (old `Color::White`), `INACTIVE_STEP` (old `Color::DarkGray`), etc. The semantic aliases preserve the same names, so most imports should work. Fix any that reference removed names.

- [ ] **Step 3: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: compiles cleanly

- [ ] **Step 4: Commit**

```bash
git add src/ui/theme.rs
git commit -m "feat: replace theme with hardware/synthwave color palette"
```

---

### Task 3: Add synth collapse state and F2 keybinding

**Files:**
- Modify: `src/app.rs` (add `synth_collapsed: bool` to `UiState`)
- Modify: `src/keys.rs` (add F2 handler)
- Modify: `src/ui/mod.rs` (branch layout on `synth_collapsed`)

- [ ] **Step 1: Add `synth_collapsed` field to `UiState` in `src/app.rs`**

Find the `UiState` struct. Add after the `show_waveform` field:
```rust
pub synth_collapsed: bool,
```

In the `UiState` default/new initialization, add:
```rust
synth_collapsed: false,
```

- [ ] **Step 2: Add F2 keybinding in `src/keys.rs`**

In the global key handler (before the focus-specific match), add:
```rust
KeyCode::F(2) => {
    app.ui.synth_collapsed = !app.ui.synth_collapsed;
}
```

- [ ] **Step 3: Branch layout in `src/ui/mod.rs` render function**

In the `render()` function, use `app.ui.synth_collapsed` to switch between expanded and collapsed synth height in the layout constraints. When collapsed, use `SYNTH_COLLAPSED_HEIGHT` instead of the full synth section height.

- [ ] **Step 4: Update `mouse.rs` layout_chunks to use synth_collapsed**

The `layout_chunks()` function must also branch on `synth_collapsed` to match the render layout.

- [ ] **Step 5: Build, test, commit**

Run: `cargo build && cargo test`

```bash
git add src/app.rs src/keys.rs src/ui/mod.rs src/mouse.rs
git commit -m "feat: add collapsible synth section with F2 toggle"
```

---

## Chunk 2: Transport Bar Redesign

### Task 4: Redesign transport bar rendering

**Files:**
- Modify: `src/ui/transport_bar.rs` (rewrite render_transport)

- [ ] **Step 1: Rewrite `render_transport()` with new layout**

The new transport bar has 4 content lines + borders:
1. Play state (cyan `▶ PLAY`) + BPM + Beat LEDs + Swing + Record (pink `● REC`)
2. `Synth  Pattern: [q] w e ... Kit: [1] 2 3 ...  Loop [ON] S:32`
3. `Drum   Pattern: [q] w e ... Kit: [1] 2 3 ...  Loop [ON] D:16`
4. `VOL ████░░  CMP ██░░  SAT ░░░░`

Use heavy-weight box drawing `┏━┓┗━┛┃` for borders. Title in border: `┏━━ TextStep - {name}* ━━...━┓`

Color assignments:
- Play/Pause icon + text: `theme::TRANSPORT_STATE` (cyan)
- Stop: `Color::Red`
- Record indicator: `theme::RECORD_COLOR` (pink)
- Active pattern/kit: `theme::AMBER` background
- Queued pattern: `theme::QUEUED_BG` (gold) background
- BPM value: `theme::TEXT` (white)
- Gauge fills: `theme::AMBER`
- Labels: `theme::DIM_TEXT`

Both Synth and Drum rows show independent pattern selectors, kit selectors, and loop indicators.

- [ ] **Step 2: Build and verify visually**

Run: `cargo run` — verify transport renders with new colors and both instrument rows.

- [ ] **Step 3: Commit**

```bash
git add src/ui/transport_bar.rs
git commit -m "feat: redesign transport bar with hardware/synthwave theme"
```

---

## Chunk 3: Drum Grid Redesign

### Task 5: New step grid with spaced squares

**Files:**
- Modify: `src/ui/drum_grid.rs` (rewrite render_drum_grid)

- [ ] **Step 1: Rewrite step rendering to use ■/□ with spacing**

Replace the current `█`/`·` packed rendering with spaced `■ □` format:
- Use `theme::STEP_ACTIVE` (■) and `theme::STEP_INACTIVE` (□)
- Each step occupies 2 characters: symbol + space
- Downbeat positions (step 0, 4, 8, 12 in each bar) use `ACTIVE_STEP_DOWNBEAT` / `INACTIVE_STEP_DOWNBEAT`
- Bar separator `┃` between step 15 and 16
- Header row: `1 · · · 2 · · · 3 · · · 4 · · ·`

- [ ] **Step 2: Add selected track indicator**

The selected track (from `app.ui.drum_ctrl_track`) should be highlighted:
- Track name rendered in `theme::SELECTED_TRACK` (pink) with `>` prefix
- Other track names in `theme::DIM_TEXT`

- [ ] **Step 3: Add breathing room**

Add empty rows:
- 1 empty row after header (between header and first track)
- 1 empty row after last track (between tracks and slider panel)

- [ ] **Step 4: Use heavy-weight borders**

Replace `Block::default().borders(Borders::ALL)` with custom heavy-weight border rendering using `theme::BOX_*` constants, or use ratatui's `BorderType::Thick`.

Title format: `DRUM MACHINE ┃ P{n}: {name}`

- [ ] **Step 5: Build and verify**

Run: `cargo run` — verify grid displays correctly with new characters and spacing.

- [ ] **Step 6: Commit**

```bash
git add src/ui/drum_grid.rs
git commit -m "feat: redesign drum grid with spaced squares and hardware borders"
```

---

### Task 6: Replace drum knobs with vertical sliders

**Files:**
- Modify: `src/ui/knobs.rs` (rewrite render_knobs → render_sliders)

- [ ] **Step 1: Rewrite knobs.rs as vertical slider panel**

Replace the current 2-row horizontal gauge layout with vertical sliders:

```
  Tune   Sweep  Color  Snap   Filter  Drive  Decay  Volume  RevSnd  DlySnd  [M] [S]
   ░      ░      ░      ░      ░      ░      ░      ░       ░       ░
   ░      ░      █      ░      ░      █      ░      ░       ░       ░
   ░      █      █      ░      █      █      ░      █       ░       ░
   █      █      █      ░      █      █      █      █       ░       ░
   █      █      █      █      █      █      █      █       █       █
  .30    .60    .20    .50    .70    .20    .50    .80     .00     .00
```

Each slider column: full-word label at top, 5 rows of `█`/`░`, value at bottom.
- Bar fill color: `theme::AMBER` (or `theme::PINK` for the currently selected param)
- Empty fill: `theme::SURFACE`
- Labels: `theme::DIM_TEXT`
- Values: `theme::AMBER`
- [M]/[S] buttons use `theme::MUTED_COLOR`/`theme::SOLOED_COLOR` when active

- [ ] **Step 2: Update `DRUM_SLIDERS_HEIGHT` in layout.rs if needed**

The slider panel needs: 1 (label) + 5 (bars) + 1 (values) + 2 (border) = 9 rows.

- [ ] **Step 3: Build, verify, commit**

```bash
git add src/ui/knobs.rs src/ui/layout.rs
git commit -m "feat: replace drum knobs with vertical slider panel"
```

---

## Chunk 4: Synth Section Redesign

### Task 7: ADSR curve rendering helper

**Files:**
- Create: `src/ui/adsr.rs`
- Modify: `src/ui/mod.rs` (add `pub mod adsr;`)

- [ ] **Step 1: Create `src/ui/adsr.rs`**

Implement an ADSR curve renderer that draws the envelope shape:

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use crate::ui::theme;

/// Render an ADSR curve visualization in the given area.
/// Area should be at least 16 wide and 6 tall.
/// Layout: 4 rows curve + 1 row values + 1 row labels
pub fn render_adsr_curve(
    f: &mut Frame,
    area: Rect,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    selected_param: Option<usize>, // 0=A, 1=D, 2=S, 3=R — highlights that column
) {
    // ... implementation
}
```

The curve uses `╱`, `╲`, `─` characters in `theme::DIM_TEXT` color.
Values shown in `theme::AMBER`, labels (A D S R) in `theme::DIM_TEXT`.

The curve shape is computed from the 4 values:
- Attack: slope height proportional to area height (always peaks at top)
- Decay: drops from peak to sustain level
- Sustain: horizontal line at sustain level
- Release: drops from sustain to zero

- [ ] **Step 2: Add `pub mod adsr;` to `src/ui/mod.rs`**

- [ ] **Step 3: Write a simple visual test**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add src/ui/adsr.rs src/ui/mod.rs
git commit -m "feat: add ADSR curve visualization helper"
```

---

### Task 8: Redesign synth knobs with grouped layout

**Files:**
- Modify: `src/ui/synth_knobs.rs` (rewrite render_synth_knobs)

- [ ] **Step 1: Rewrite synth_knobs as grouped vertical sliders + ADSR curves**

Organize into labeled groups with `╶── LABEL ──╴` dividers:

**OSC1 group**: Wave selector `[Sqr] Saw Sin Rnd` + Tune/PWM/Level sliders
**OSC2 group**: Wave selector + Tune/PWM/Level/Detune/Sub sliders
**ENV1 group**: ADSR curve (using `adsr::render_adsr_curve`)
**ENV2 group**: ADSR curve
**FILT group**: Type selector `[LP] HP BP` + Freq/Res/EnvAmt sliders + FILT ADSR curve
**AMP group**: Vol/Reverb/Delay/Sat sliders

Wave/Type selectors: active option in `theme::AMBER` with brackets, others in `theme::DIM_TEXT`.

- [ ] **Step 2: Build and verify**

Run: `cargo run` — verify synth params render with curves and grouped layout.

- [ ] **Step 3: Commit**

```bash
git add src/ui/synth_knobs.rs
git commit -m "feat: redesign synth params with ADSR curves and grouped sliders"
```

---

### Task 9: Redesign synth grid with consistent step format

**Files:**
- Modify: `src/ui/synth_grid.rs`

- [ ] **Step 1: Update synth grid to use same ■/□ step format as drums**

- Same header row format: `1 · · · 2 · · · 3 · · · 4 · · ·`
- Same spaced square characters
- Same bar separator `┃`
- Same downbeat differentiation

- [ ] **Step 2: Build, verify, commit**

```bash
git add src/ui/synth_grid.rs
git commit -m "feat: redesign synth grid with consistent step format"
```

---

### Task 10: Implement collapsed synth rendering

**Files:**
- Modify: `src/ui/mod.rs` (update synth section rendering)

- [ ] **Step 1: When `synth_collapsed`, render only the step row**

Show: `┏━ SYNTH [F2 expand] ━━━━...━┓` with just the single step row inside and `┗━━━━...━┛` below.

- [ ] **Step 2: Build, verify, commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: implement collapsed synth section rendering"
```

---

## Chunk 5: Mouse Interaction Improvements

### Task 11: Click-to-focus on sections

**Files:**
- Modify: `src/mouse.rs`

- [ ] **Step 1: Add section-level hit testing**

In `handle_left_down()`, before checking specific elements, test if the click falls within a section's bounding rect:
- If click is within transport area → set `app.ui.focus = FocusSection::Transport`
- If click is within synth area → set focus to `SynthGrid` or `SynthControls`
- If click is within drum area → set focus to `DrumGrid` or `Knobs`

Use the layout_chunks rects already computed.

- [ ] **Step 2: Build, verify, commit**

```bash
git add src/mouse.rs
git commit -m "feat: add click-to-focus on bordered sections"
```

---

### Task 12: Clickable transport controls

**Files:**
- Modify: `src/mouse.rs`

- [ ] **Step 1: Add transport click hit zones**

In the transport area click handler, add hit zones for:
- Play/Stop text area (columns ~2-9, row 0): toggle play/pause
- REC indicator (last ~5 columns, row 0): toggle record
- Pattern letters (row 1-2): queue pattern (reuse existing pattern selector click logic)
- Kit numbers (row 1-2): switch kit

- [ ] **Step 2: Build, verify, commit**

```bash
git add src/mouse.rs
git commit -m "feat: add clickable transport controls"
```

---

### Task 13: Mouse wheel support for parameters

**Files:**
- Modify: `src/mouse.rs`

- [ ] **Step 1: Handle `MouseEventKind::ScrollUp` and `ScrollDown`**

In the mouse event handler, add cases for scroll events:
- Over a drum slider: adjust param by ±0.01
- Over a synth slider: adjust param by ±0.01
- Over a gauge in transport (CMP, VOL, SAT): adjust by ±0.01
- Over BPM display: adjust by ±1.0
- Over wave/type selector: cycle options

- [ ] **Step 2: Build, verify, commit**

```bash
git add src/mouse.rs
git commit -m "feat: add mouse wheel support for parameter adjustment"
```

---

### Task 14: Update mouse hit-testing for new layout

**Files:**
- Modify: `src/mouse.rs`

- [ ] **Step 1: Update grid hit-testing for spaced squares**

The new step format uses 2 chars per step (symbol + space) instead of the old format. Update the column calculation in `hit_test_grid_step()` to account for the new spacing.

- [ ] **Step 2: Update slider hit-testing**

Replace the old inline gauge/knob hit zones with vertical slider hit zones. Each slider column occupies ~7 chars (label width). Map click X position to slider index, Y position to drag.

- [ ] **Step 3: Extensive manual testing**

Test at multiple terminal sizes. Verify:
- Clicking steps at all positions works correctly
- Clicking sliders selects correct param
- Resize doesn't break hit testing

- [ ] **Step 4: Commit**

```bash
git add src/mouse.rs
git commit -m "feat: update mouse hit-testing for new layout and slider format"
```

---

## Chunk 6: Polish & Fine-Tuning

### Task 15: Finer keyboard parameter increments

**Files:**
- Modify: `src/keys.rs`

- [ ] **Step 1: Define increment constants**

At the top of `keys.rs`, add:
```rust
const PARAM_INCREMENT: f32 = 0.02;
```

- [ ] **Step 2: Replace all hardcoded 0.05 increments**

Search for `0.05` in keys.rs and replace with `PARAM_INCREMENT`. Also search for `-0.05`.

- [ ] **Step 3: Build, test, commit**

```bash
git add src/keys.rs
git commit -m "feat: finer parameter increments (0.02 per step)"
```

---

### Task 16: Update activity bar for new theme

**Files:**
- Modify: `src/ui/mod.rs` (render_activity_bar function)

- [ ] **Step 1: Update activity bar colors**

- Pad labels: `theme::DIM_TEXT` normally, `theme::TEXT` on `theme::AMBER` bg when flashing
- Separator: `theme::BORDER`
- Param display: `theme::AMBER` for values
- Help hint: `theme::DIM_TEXT`

- [ ] **Step 2: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: update activity bar for new theme"
```

---

### Task 17: Update modal dialogs for new theme

**Files:**
- Modify: `src/ui/mod.rs` (render_text_input, render_file_picker, render_preset_browser, render_pattern_browser)

- [ ] **Step 1: Update modal colors**

- Border: `theme::AMBER` (was yellow)
- Title: `theme::AMBER` bold
- Selected item: `theme::CYAN` bg (was cyan — stays similar)
- Text: `theme::TEXT`
- Hints: `theme::DIM_TEXT`

- [ ] **Step 2: Commit**

```bash
git add src/ui/mod.rs
git commit -m "feat: update modal dialogs for new theme"
```

---

### Task 18: Final integration and visual QA

**Files:**
- All UI files

- [ ] **Step 1: Full build**

Run: `cargo build --release 2>&1 | tail -5`
Expected: clean build, no warnings

- [ ] **Step 2: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: all tests pass

- [ ] **Step 3: Visual QA checklist**

Run the app and verify:
- [ ] Transport bar: both Synth/Drum rows, clickable play, colored accents
- [ ] Synth expanded: ADSR curves render, wave selectors work, all params grouped
- [ ] Synth collapsed: F2 toggles, only step row visible
- [ ] Drum grid: spaced squares, bar separator, selected track pink, downbeat colors
- [ ] Drum sliders: vertical bars, full-word labels, mouse wheel adjusts
- [ ] Click-to-focus: clicking sections changes border color
- [ ] Mouse: no offset on resize, scroll wheel works on params
- [ ] Splash + Help: unchanged, still work
- [ ] Waveform toggle: still works
- [ ] Modal dialogs: new color scheme

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete TUI redesign — hardware/synthwave aesthetic"
```
