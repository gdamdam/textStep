// Bar-style scope display and VU meter — 90s Hi-Fi LED aesthetic

use std::sync::Arc;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::audio::display_buffer::AudioDisplayBuffer;

// Hi-Fi LED base colors (full brightness)
const LED_GREEN: (u8, u8, u8) = (0, 220, 0);
const LED_YELLOW: (u8, u8, u8) = (220, 220, 0);
const LED_ORANGE: (u8, u8, u8) = (255, 140, 0);
const LED_RED: (u8, u8, u8) = (255, 30, 0);

// Dim "off" tints for unlit LEDs
const LED_GREEN_OFF: (u8, u8, u8) = (0, 30, 0);
const LED_YELLOW_OFF: (u8, u8, u8) = (30, 30, 0);
const LED_ORANGE_OFF: (u8, u8, u8) = (35, 20, 0);
const LED_RED_OFF: (u8, u8, u8) = (40, 5, 0);

/// Blend a color between dim and full based on brightness (0.0-1.0).
fn led_color(base: (u8, u8, u8), dim: (u8, u8, u8), brightness: f32) -> Color {
    let b = brightness.clamp(0.0, 1.0);
    let r = dim.0 as f32 + (base.0 as f32 - dim.0 as f32) * b;
    let g = dim.1 as f32 + (base.1 as f32 - dim.1 as f32) * b;
    let bl = dim.2 as f32 + (base.2 as f32 - dim.2 as f32) * b;
    Color::Rgb(r as u8, g as u8, bl as u8)
}

/// Pick the Hi-Fi color zone based on row position (0.0 = bottom, 1.0 = top).
fn hifi_zone(ratio: f32) -> ((u8, u8, u8), (u8, u8, u8)) {
    if ratio > 0.78 {
        (LED_RED, LED_RED_OFF)
    } else if ratio > 0.56 {
        (LED_ORANGE, LED_ORANGE_OFF)
    } else if ratio > 0.33 {
        (LED_YELLOW, LED_YELLOW_OFF)
    } else {
        (LED_GREEN, LED_GREEN_OFF)
    }
}

/// Render the VU meter as a vertical bar — Hi-Fi LED style.
pub fn render_vu_meter(f: &mut Frame, area: Rect, display_buf: &Arc<AudioDisplayBuffer>) {
    let peak = display_buf.get_peak();
    let level = (peak.sqrt() * 9.0).clamp(0.0, 9.0);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(30, 30, 30)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    for row_idx in 0..inner.height.min(9) {
        let bar_level = 8 - row_idx as usize; // 8=top, 0=bottom
        let ratio = bar_level as f32 / 8.0;
        let (base, dim) = hifi_zone(ratio);

        let is_lit = (bar_level as f32) < level;
        // Brightness proportional to how far above the segment we are
        let brightness = if is_lit {
            ((level - bar_level as f32) / 1.0).clamp(0.3, 1.0)
        } else {
            0.0
        };

        let color = if is_lit {
            led_color(base, dim, brightness)
        } else {
            Color::Rgb(dim.0, dim.1, dim.2)
        };

        let ch = if is_lit { "\u{2588}" } else { "\u{2591}" };
        let span = Span::styled(ch, Style::default().fg(color));
        let line = Line::from(span);
        let row_area = Rect::new(inner.x, inner.y + row_idx, inner.width.min(1), 1);
        f.render_widget(Paragraph::new(line), row_area);
    }
}

/// Unicode block characters for sub-row resolution (8ths from bottom).
const BAR_CHARS: [char; 9] = [
    ' ',
    '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}',
    '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}',
];

/// Frequency markers: label + frequency in Hz.
/// Logarithmically spaced from 30Hz to 20kHz.
const FREQ_MARKERS: &[(f32, &str)] = &[
    (40.0, "40"),
    (100.0, "100"),
    (250.0, "250"),
    (500.0, "500"),
    (1000.0, "1k"),
    (2500.0, "2.5k"),
    (5000.0, "5k"),
    (10000.0, "10k"),
    (20000.0, "20k"),
];

const FREQ_LO: f32 = 40.0;
const FREQ_HI: f32 = 20000.0;

/// Map a frequency to a column position (0.0-1.0) using logarithmic scale.
fn freq_to_col_ratio(freq: f32) -> f32 {
    let log_lo = FREQ_LO.ln();
    let log_hi = FREQ_HI.ln();
    (freq.ln() - log_lo) / (log_hi - log_lo)
}

/// Render the spectrum analyzer as vertical bars with frequency labels.
/// `scope_bars` = peak-held bar heights, `scope_intensity` = brightness per bar.
pub fn render_scope_bars(f: &mut Frame, area: Rect, scope_bars: &[f32], scope_intensity: &[f32]) {
    let block = Block::default()
        .title(" Spectrum ")
        .title_style(Style::default().fg(Color::Rgb(60, 60, 60)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(30, 30, 30)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 10 || inner.height < 3 || scope_bars.is_empty() {
        return;
    }

    let width = inner.width as usize;
    // Reserve bottom row for frequency labels
    let bar_height = (inner.height as usize).saturating_sub(1);
    let num_bars = scope_bars.len();

    if bar_height < 2 {
        return;
    }

    const MIN_BRIGHT: f32 = 0.25;

    // Build the bar grid (bar_height rows) — unlit cells show dim LED tints (90s Hi-Fi look)
    let mut grid = vec![vec![(' ', Color::Rgb(8, 15, 8), Color::Reset); width]; bar_height];

    // Pre-fill with dim "off" LED background per zone (like VU meter unlit segments)
    for col in 0..width {
        for row in 0..bar_height {
            let row_from_bottom = bar_height - 1 - row;
            let ratio = row_from_bottom as f32 / bar_height.max(1) as f32;
            let (_base, dim) = hifi_zone(ratio);
            let bg = Color::Rgb(dim.0 / 2, dim.1 / 2, dim.2 / 2);
            grid[row][col] = ('\u{2591}', Color::Rgb(dim.0, dim.1, dim.2), bg);
        }
    }

    for col in 0..width {
        // Fractional bar index for smooth interpolation between bands
        let frac_idx = col as f32 * (num_bars - 1) as f32 / (width - 1).max(1) as f32;
        let idx_lo = (frac_idx as usize).min(num_bars - 1);
        let idx_hi = (idx_lo + 1).min(num_bars - 1);
        let t = frac_idx - idx_lo as f32; // interpolation factor 0.0-1.0

        let level = (scope_bars[idx_lo] * (1.0 - t) + scope_bars[idx_hi] * t).clamp(0.0, 1.0);
        let int_lo = scope_intensity.get(idx_lo).copied().unwrap_or(0.5);
        let int_hi = scope_intensity.get(idx_hi).copied().unwrap_or(0.5);
        let intensity = int_lo * (1.0 - t) + int_hi * t;
        let brightness = MIN_BRIGHT + (1.0 - MIN_BRIGHT) * intensity;

        let total_sub = (level * (bar_height * 8) as f32) as usize;
        let full_rows = total_sub / 8;
        let remainder = total_sub % 8;

        for row in 0..bar_height {
            let row_from_bottom = bar_height - 1 - row;
            let ratio = row_from_bottom as f32 / bar_height.max(1) as f32;
            let (base, dim) = hifi_zone(ratio);
            let bg = Color::Rgb(dim.0 / 2, dim.1 / 2, dim.2 / 2);

            if row_from_bottom < full_rows {
                let color = led_color(base, dim, brightness);
                grid[row][col] = ('\u{2588}', color, bg);
            } else if row_from_bottom == full_rows && remainder > 0 {
                let color = led_color(base, dim, brightness);
                grid[row][col] = (BAR_CHARS[remainder], color, bg);
            }
        }
    }

    // Render bar rows
    for (row_idx, row_chars) in grid.iter().enumerate() {
        let spans: Vec<Span> = row_chars.iter().map(|&(ch, fg, bg)| {
            Span::styled(ch.to_string(), Style::default().fg(fg).bg(bg))
        }).collect();
        let row_area = Rect::new(inner.x, inner.y + row_idx as u16, inner.width, 1);
        f.render_widget(Paragraph::new(Line::from(spans)), row_area);
    }

    // Render frequency label row at the bottom
    let label_y = inner.y + bar_height as u16;
    let mut label_row = vec![(' ', Color::Rgb(40, 40, 40)); width];

    // Place tick marks and labels
    for &(freq, label) in FREQ_MARKERS {
        let ratio = freq_to_col_ratio(freq);
        let col = (ratio * (width - 1) as f32).round() as usize;
        if col >= width { continue; }

        // Place the label centered on the tick position
        let label_chars: Vec<char> = label.chars().collect();
        let label_len = label_chars.len();
        let start = col.saturating_sub(label_len / 2);

        for (i, &ch) in label_chars.iter().enumerate() {
            let c = start + i;
            if c < width {
                label_row[c] = (ch, Color::Rgb(100, 100, 100));
            }
        }
    }

    let label_bg = Color::Rgb(0, 15, 0);
    let label_spans: Vec<Span> = label_row.iter().map(|&(ch, color)| {
        Span::styled(ch.to_string(), Style::default().fg(color).bg(label_bg))
    }).collect();
    let label_area = Rect::new(inner.x, label_y, inner.width, 1);
    f.render_widget(Paragraph::new(Line::from(label_spans)), label_area);
}
