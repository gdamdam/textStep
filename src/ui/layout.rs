// src/ui/layout.rs — Single source of truth for all layout dimensions

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Transport bar height (title border + 4 content lines + bottom border)
pub const TRANSPORT_HEIGHT: u16 = 6;

/// Height of the drum knobs panel (1 label + 5 bars + 1 value + 2 border).
pub const KNOBS_HEIGHT: u16 = 9;

/// Height of the synth knobs panel (OSC 8 + ENV/FILT 8 + LFO 3 + AMP 7 + 2 border = 30).
pub const SYNTH_KNOBS_HEIGHT: u16 = 30;

/// Height of the synth step row (2 border + header + spacer + step row + spacer = 6).
pub const SYNTH_GRID_HEIGHT: u16 = 6;

/// Combined synth section height (knobs + steps).
pub const SYNTH_SECTION_HEIGHT: u16 = SYNTH_KNOBS_HEIGHT + SYNTH_GRID_HEIGHT;

/// Synth section when collapsed (border + step row + border)
pub const SYNTH_COLLAPSED_HEIGHT: u16 = 3;

/// Width of the volume fader column.
pub const FADER_WIDTH: u16 = 3;

/// Height of the waveform/oscilloscope panel (including borders).
pub const WAVEFORM_HEIGHT: u16 = 11;

/// Activity bar (bottom status line)
pub const ACTIVITY_BAR_HEIGHT: u16 = 1;

/// Separator line
pub const SEPARATOR_HEIGHT: u16 = 1;

/// Help panel height
pub const HELP_HEIGHT: u16 = 22;

/// Pre-computed layout rects, shared between render and mouse hit-testing.
pub struct ComputedLayout {
    pub transport: Rect,
    pub synth_section: Rect,
    pub separator: Rect,
    pub drum_area: Rect,
    pub knobs: Rect,
    pub extra: Option<Rect>,  // Help or Waveform panel
    pub activity_bar: Rect,
    // Synth sub-areas (only valid when expanded)
    pub synth_fader: Rect,
    pub synth_content: Rect,
    pub synth_knobs: Rect,
    pub synth_grid: Rect,
    // Drum sub-areas
    pub drum_fader: Rect,
    pub drum_grid: Rect,
}

/// Compute layout for all sections. Both ui/mod.rs and mouse.rs consume this.
pub fn compute_layout(
    size: Rect,
    synth_collapsed: bool,
    show_help: bool,
    show_waveform: bool,
) -> ComputedLayout {
    let synth_height = if synth_collapsed { SYNTH_COLLAPSED_HEIGHT } else { SYNTH_SECTION_HEIGHT };

    let chunks = if show_help {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TRANSPORT_HEIGHT),
                Constraint::Length(synth_height),
                Constraint::Length(SEPARATOR_HEIGHT),
                Constraint::Min(11),
                Constraint::Length(KNOBS_HEIGHT),
                Constraint::Length(HELP_HEIGHT),
                Constraint::Length(ACTIVITY_BAR_HEIGHT),
            ])
            .split(size)
    } else if show_waveform {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TRANSPORT_HEIGHT),
                Constraint::Length(synth_height),
                Constraint::Length(SEPARATOR_HEIGHT),
                Constraint::Min(11),
                Constraint::Length(KNOBS_HEIGHT),
                Constraint::Length(WAVEFORM_HEIGHT),
                Constraint::Length(ACTIVITY_BAR_HEIGHT),
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(TRANSPORT_HEIGHT),
                Constraint::Length(synth_height),
                Constraint::Length(SEPARATOR_HEIGHT),
                Constraint::Min(11),
                Constraint::Length(KNOBS_HEIGHT),
                Constraint::Length(ACTIVITY_BAR_HEIGHT),
            ])
            .split(size)
    };

    let extra = if show_help || show_waveform { Some(chunks[5]) } else { None };
    let activity_idx = if show_help || show_waveform { 6 } else { 5 };

    let synth_section = chunks[1];

    // Synth sub-splits
    let synth_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(FADER_WIDTH), Constraint::Min(20)])
        .split(synth_section);
    let synth_fader = synth_h[0];
    let synth_content = synth_h[1];

    let synth_v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(SYNTH_KNOBS_HEIGHT), Constraint::Length(SYNTH_GRID_HEIGHT)])
        .split(synth_content);
    let synth_knobs = synth_v[0];
    let synth_grid = synth_v[1];

    // Drum sub-splits
    let drum_area = chunks[3];
    let drum_h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(FADER_WIDTH), Constraint::Min(20)])
        .split(drum_area);
    let drum_fader = drum_h[0];
    let drum_grid = drum_h[1];

    ComputedLayout {
        transport: chunks[0],
        synth_section,
        separator: chunks[2],
        drum_area,
        knobs: chunks[4],
        extra,
        activity_bar: chunks[activity_idx],
        synth_fader,
        synth_content,
        synth_knobs,
        synth_grid,
        drum_fader,
        drum_grid,
    }
}
