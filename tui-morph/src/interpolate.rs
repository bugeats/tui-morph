use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::oklch::{self, Oklch};
use crate::plan::{ColorPair, InterpolationPlan};

const LEGIBILITY_THRESHOLD: f32 = 0.15;

/// `t` must be in `[0.0, 1.0]`.
pub fn render(plan: &InterpolationPlan, t: f32) -> Buffer {
    let area = Rect::new(0, 0, plan.width, plan.height);
    let mut buf = Buffer::empty(area);

    render_stable(plan, &mut buf);
    render_mutating(plan, t, &mut buf);
    render_displaced(plan, t, &mut buf);
    render_appearing(plan, t, &mut buf);
    render_disappearing(plan, t, &mut buf);

    buf
}

fn render_stable(plan: &InterpolationPlan, buf: &mut Buffer) {
    for cell in &plan.stable {
        let target = &mut buf[(cell.x, cell.y)];
        target.set_symbol(&cell.symbol);
        target.set_style(Style::new().fg(cell.fg).bg(cell.bg));
        target.modifier = cell.modifier;
    }
}

fn render_mutating(plan: &InterpolationPlan, t: f32, buf: &mut Buffer) {
    for cell in &plan.mutating {
        let fg = lerp_color(&cell.src_fg, &cell.dst_fg, t);
        let bg = lerp_color(&cell.src_bg, &cell.dst_bg, t);
        let symbol = pick_symbol(&cell.src_symbol, &cell.dst_symbol, &cell.src_fg, t);
        let modifier = if t < 0.5 {
            cell.src_modifier
        } else {
            cell.dst_modifier
        };

        let target = &mut buf[(cell.x, cell.y)];
        target.set_symbol(&symbol);
        target.set_style(Style::new().fg(fg).bg(bg));
        target.modifier = modifier;
    }
}

fn render_displaced(plan: &InterpolationPlan, t: f32, buf: &mut Buffer) {
    for cell in &plan.displaced {
        let x = lerp_pos(cell.src_x, cell.dst_x, t);
        let y = lerp_pos(cell.src_y, cell.dst_y, t);

        if x >= plan.width || y >= plan.height {
            continue;
        }

        let fg = lerp_color(&cell.src_fg, &cell.dst_fg, t);
        let bg = lerp_color(&cell.src_bg, &cell.dst_bg, t);
        let symbol = pick_symbol(&cell.src_symbol, &cell.dst_symbol, &cell.src_fg, t);
        let modifier = if t < 0.5 {
            cell.src_modifier
        } else {
            cell.dst_modifier
        };

        let target = &mut buf[(x, y)];
        target.set_symbol(&symbol);
        target.set_style(Style::new().fg(fg).bg(bg));
        target.modifier = modifier;
    }
}

fn render_appearing(plan: &InterpolationPlan, t: f32, buf: &mut Buffer) {
    for cell in &plan.appearing {
        let factor = t;
        let fg = fade(&cell.fg, factor);
        let bg = lerp_color(&cell.counter_bg, &cell.bg, t);

        let visible = cell
            .fg
            .oklch
            .map(|lch| lch.l * factor >= LEGIBILITY_THRESHOLD)
            .unwrap_or(factor >= 0.5);

        let target = &mut buf[(cell.x, cell.y)];
        target.set_style(Style::new().fg(fg).bg(bg));

        if visible {
            target.set_symbol(&cell.symbol);
            target.modifier = cell.modifier;
        }
    }
}

fn render_disappearing(plan: &InterpolationPlan, t: f32, buf: &mut Buffer) {
    for cell in &plan.disappearing {
        let factor = 1.0 - t;
        let fg = fade(&cell.fg, factor);
        let bg = lerp_color(&cell.bg, &cell.counter_bg, t);

        let visible = cell
            .fg
            .oklch
            .map(|lch| lch.l * factor >= LEGIBILITY_THRESHOLD)
            .unwrap_or(factor >= 0.5);

        let target = &mut buf[(cell.x, cell.y)];
        target.set_style(Style::new().fg(fg).bg(bg));

        if visible {
            target.set_symbol(&cell.symbol);
            target.modifier = cell.modifier;
        }
    }
}

fn lerp_color(src: &ColorPair, dst: &ColorPair, t: f32) -> ratatui::style::Color {
    match (src.oklch, dst.oklch) {
        (Some(a), Some(b)) => oklch::to_color(oklch::lerp(a, b, t)),
        _ if t < 0.5 => src.raw,
        _ => dst.raw,
    }
}

/// Crossfade through black: old glyph fades to invisible, new glyph emerges.
fn pick_symbol<'a>(src: &'a str, dst: &'a str, src_fg: &ColorPair, t: f32) -> &'a str {
    if src == dst {
        return src;
    }

    let threshold = src_fg
        .oklch
        .map(|lch| {
            if lch.l < 0.01 {
                0.5
            } else {
                (LEGIBILITY_THRESHOLD / lch.l).clamp(0.0, 1.0)
            }
        })
        .unwrap_or(0.5);

    if t < threshold {
        src
    } else {
        dst
    }
}

/// Scale lightness by `factor` (0.0 = black, 1.0 = original).
fn fade(color: &ColorPair, factor: f32) -> ratatui::style::Color {
    match color.oklch {
        Some(lch) => oklch::to_color(Oklch {
            l: lch.l * factor,
            ..lch
        }),
        None if factor >= 0.5 => color.raw,
        None => ratatui::style::Color::Reset,
    }
}

fn lerp_pos(src: u16, dst: u16, t: f32) -> u16 {
    let s = src as f32;
    let d = dst as f32;
    (s + (d - s) * t).round() as u16
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

    use crate::solver;
    use crate::weights::MorphWeights;

    use super::*;

    fn make_buffer(width: u16, height: u16, cells: &[((u16, u16), &str, Color)]) -> Buffer {
        let area = Rect::new(0, 0, width, height);
        let mut buf = Buffer::empty(area);

        for &((x, y), sym, fg) in cells {
            let cell = &mut buf[(x, y)];
            cell.set_symbol(sym);
            cell.set_style(Style::default().fg(fg));
        }

        buf
    }

    #[test]
    fn at_zero_matches_source() {
        let src = make_buffer(3, 1, &[((0, 0), "A", Color::Red), ((1, 0), "B", Color::Blue)]);
        let dst = make_buffer(3, 1, &[((0, 0), "X", Color::Green), ((1, 0), "Y", Color::White)]);
        let plan = solver::diff(&src, &dst, &MorphWeights::CRISP);
        let result = render(&plan, 0.0);

        assert_eq!(result[(0, 0)].symbol(), "A");
        assert_eq!(result[(1, 0)].symbol(), "B");
    }

    #[test]
    fn at_one_matches_target() {
        let src = make_buffer(3, 1, &[((0, 0), "A", Color::Red), ((1, 0), "B", Color::Blue)]);
        let dst = make_buffer(3, 1, &[((0, 0), "X", Color::Green), ((1, 0), "Y", Color::White)]);
        let plan = solver::diff(&src, &dst, &MorphWeights::CRISP);
        let result = render(&plan, 1.0);

        assert_eq!(result[(0, 0)].symbol(), "X");
        assert_eq!(result[(1, 0)].symbol(), "Y");
    }

    #[test]
    fn stable_cells_unchanged_at_any_t() {
        let src = make_buffer(2, 1, &[((0, 0), "S", Color::Red)]);
        let dst = make_buffer(2, 1, &[((0, 0), "S", Color::Red)]);
        let plan = solver::diff(&src, &dst, &MorphWeights::LIQUID);

        for i in 0..=10 {
            let t = i as f32 / 10.0;
            let result = render(&plan, t);
            assert_eq!(result[(0, 0)].symbol(), "S");
            assert_eq!(result[(0, 0)].fg, Color::Red);
        }
    }

    #[test]
    fn color_interpolation_midpoint() {
        let src = make_buffer(1, 1, &[((0, 0), "X", Color::Rgb(255, 0, 0))]);
        let dst = make_buffer(1, 1, &[((0, 0), "X", Color::Rgb(0, 0, 255))]);
        let plan = solver::diff(&src, &dst, &MorphWeights::LIQUID);
        let result = render(&plan, 0.5);

        let fg = result[(0, 0)].fg;
        match fg {
            Color::Rgb(r, _, b) => {
                assert!(r < 255 && b < 255, "expected interpolated color, got {fg:?}");
            }
            _ => panic!("expected Rgb color"),
        }
    }
}
