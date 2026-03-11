//! Drum step grid: 8 tracks x 32 steps with playhead, cursor, and mute/solo indicators.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};

use crate::app::{App, FocusSection};
use crate::sequencer::drum_pattern::{MAX_STEPS, NUM_DRUM_TRACKS, TRACK_IDS};
use crate::sequencer::transport::PlayState;
use crate::ui::theme;

/// Track name column: ">Cowbell " or " Cowbell " = 9 chars (padded to longest name)
const NAME_WIDTH: usize = 9;

/// Renders the 8-track drum grid with step indicators, track names,
/// mute/solo buttons, playhead column, and cursor highlight.
pub fn render_drum_grid(f: &mut Frame, area: Rect, app: &App) {
    let focused_grid = app.ui.focus == FocusSection::DrumGrid;
    let border_style = theme::focus_border_style(focused_grid);
    let is_playing = app.transport.state == PlayState::Playing;

    let pattern_name = app.current_pattern_name();
    let pattern_num = app.ui.active_pattern + 1;
    let block = Block::default()
        .title(format!(" DRUM MACHINE \u{2503} P{}: {} ", pattern_num, pattern_name))
        .title_style(Style::default().fg(theme::TITLE_COLOR).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(border_style);

    let mut lines: Vec<Line> = Vec::new();

    // ── Header row ───────────────────────────────────────────────
    let mut header_spans: Vec<Span> = Vec::new();
    // Pad to align with track name column
    header_spans.push(Span::styled(
        " ".repeat(NAME_WIDTH),
        Style::default().fg(theme::DIM_TEXT),
    ));

    for step in 0..MAX_STEPS {
        if step == 16 {
            header_spans.push(Span::styled(
                "\u{2503}",
                Style::default().fg(theme::BORDER),
            ));
        }
        let beat_in_bar = step % 16;
        let beat_num = beat_in_bar / 4 + 1;
        let sub = beat_in_bar % 4;
        if sub == 0 {
            header_spans.push(Span::styled(
                format!("{} ", beat_num),
                Style::default().fg(theme::DIM_TEXT),
            ));
        } else {
            header_spans.push(Span::styled(
                "\u{00B7} ",
                Style::default().fg(theme::BORDER),
            ));
        }
    }
    lines.push(Line::from(header_spans));

    // ── Spacer row between header and first track ────────────────
    lines.push(Line::from(""));

    // ── Track rows ───────────────────────────────────────────────
    for track in 0..NUM_DRUM_TRACKS {
        let mut spans: Vec<Span> = Vec::new();
        let track_name = TRACK_IDS[track].name();
        let is_ctrl_track = app.ui.drum_ctrl_track == track;

        // Track name with selection indicator
        if is_ctrl_track {
            let label = format!(">{:<width$}", track_name, width = NAME_WIDTH - 1);
            spans.push(Span::styled(
                label,
                Style::default().fg(theme::PINK).add_modifier(Modifier::BOLD),
            ));
        } else {
            let label = format!(" {:<width$}", track_name, width = NAME_WIDTH - 1);
            spans.push(Span::styled(
                label,
                Style::default().fg(theme::DIM_TEXT),
            ));
        }

        // Steps 0-31
        for step in 0..MAX_STEPS {
            if step == 16 {
                spans.push(Span::styled(
                    "\u{2503}",
                    Style::default().fg(theme::BORDER),
                ));
            }

            let active = app.drum_pattern.steps[track][step];
            let is_playhead = is_playing && app.ui.playback_step == step;
            let is_cursor = focused_grid
                && app.ui.drum_cursor_track == track
                && app.ui.drum_cursor_step == step;
            let is_downbeat = step % 4 == 0;

            let symbol = if active {
                theme::STEP_ACTIVE
            } else {
                theme::STEP_INACTIVE
            };

            // Determine base foreground color based on active/downbeat
            let base_fg = if active && is_downbeat {
                theme::AMBER_BRIGHT
            } else if active {
                theme::AMBER
            } else if is_downbeat {
                theme::AMBER_DIM
            } else {
                theme::BORDER
            };

            let style = if is_cursor && is_playhead {
                Style::default()
                    .fg(theme::CURSOR_FG)
                    .bg(theme::PLAYHEAD_BG)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else if is_cursor {
                Style::default()
                    .fg(theme::CURSOR_FG)
                    .bg(theme::CURSOR_BG)
                    .add_modifier(Modifier::BOLD)
            } else if is_playhead {
                Style::default()
                    .fg(theme::PLAYHEAD_FG)
                    .bg(theme::PLAYHEAD_BG)
            } else {
                Style::default().fg(base_fg)
            };

            spans.push(Span::styled(format!("{} ", symbol), style));
        }

        // [M] [S] buttons after steps
        let params = &app.drum_pattern.params[track];
        spans.push(Span::raw(" "));
        let mute_style = if params.mute {
            Style::default().fg(theme::MUTED_COLOR).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };
        spans.push(Span::styled("[M]", mute_style));
        spans.push(Span::raw(" "));
        let solo_style = if params.solo {
            Style::default().fg(theme::SOLOED_COLOR).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DIM_TEXT)
        };
        spans.push(Span::styled("[S]", solo_style));

        lines.push(Line::from(spans));
    }

    // ── Spacer row after last track ──────────────────────────────
    lines.push(Line::from(""));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}
