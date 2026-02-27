use ratatui::buffer::Buffer;

use crate::oklch;
use crate::plan::{
    ColorPair, DisplacedCell, InterpolationPlan, MutatingCell, OrphanCell, StableCell,
};
use crate::weights::MorphWeights;

/// Buffers must have the same dimensions.
pub fn diff(src: &Buffer, dst: &Buffer, weights: &MorphWeights) -> InterpolationPlan {
    let area = src.area();
    assert_eq!(area, dst.area(), "buffers must have the same dimensions");

    let width = area.width;
    let height = area.height;

    let mut stable = Vec::new();
    let mut mutating = Vec::new();
    let mut src_unmatched: Vec<(u16, u16, CellSnapshot)> = Vec::new();
    let mut dst_unmatched: Vec<(u16, u16, CellSnapshot)> = Vec::new();

    for y in area.y..area.y + height {
        for x in area.x..area.x + width {
            let sc = &src[(x, y)];
            let dc = &dst[(x, y)];

            let same_symbol = sc.symbol() == dc.symbol();
            let same_fg = sc.fg == dc.fg;
            let same_bg = sc.bg == dc.bg;
            let same_modifier = sc.modifier == dc.modifier;

            if same_symbol && same_fg && same_bg && same_modifier {
                stable.push(StableCell {
                    x,
                    y,
                    symbol: sc.symbol().to_string(),
                    fg: sc.fg,
                    bg: sc.bg,
                    modifier: sc.modifier,
                });
            } else if is_blank_cell(sc) && !is_blank_cell(dc) {
                dst_unmatched.push((x, y, CellSnapshot::from_cell(dc)));
            } else if !is_blank_cell(sc) && is_blank_cell(dc) {
                src_unmatched.push((x, y, CellSnapshot::from_cell(sc)));
            } else if !is_blank_cell(sc) && !is_blank_cell(dc) {
                mutating.push(MutatingCell {
                    x,
                    y,
                    src_symbol: sc.symbol().to_string(),
                    dst_symbol: dc.symbol().to_string(),
                    src_fg: ColorPair::from_color(sc.fg),
                    dst_fg: ColorPair::from_color(dc.fg),
                    src_bg: ColorPair::from_color(sc.bg),
                    dst_bg: ColorPair::from_color(dc.bg),
                    src_modifier: sc.modifier,
                    dst_modifier: dc.modifier,
                });
            } else {
                // Both blank but different style — snap to target.
                stable.push(StableCell {
                    x,
                    y,
                    symbol: dc.symbol().to_string(),
                    fg: dc.fg,
                    bg: dc.bg,
                    modifier: dc.modifier,
                });
            }
        }
    }

    let (displaced, appearing, disappearing) =
        solve_unmatched(&src_unmatched, &dst_unmatched, weights);

    InterpolationPlan {
        width,
        height,
        stable,
        mutating,
        displaced,
        appearing,
        disappearing,
    }
}

fn is_blank_cell(cell: &ratatui::buffer::Cell) -> bool {
    let sym = cell.symbol();
    sym == " " || sym.is_empty()
}

#[derive(Clone)]
struct CellSnapshot {
    symbol: String,
    fg: ColorPair,
    bg: ColorPair,
    modifier: ratatui::style::Modifier,
}

impl CellSnapshot {
    fn from_cell(cell: &ratatui::buffer::Cell) -> Self {
        Self {
            symbol: cell.symbol().to_string(),
            fg: ColorPair::from_color(cell.fg),
            bg: ColorPair::from_color(cell.bg),
            modifier: cell.modifier,
        }
    }
}

fn orphan_from(x: u16, y: u16, snap: &CellSnapshot) -> OrphanCell {
    OrphanCell {
        x,
        y,
        symbol: snap.symbol.clone(),
        fg: snap.fg,
        bg: snap.bg,
        modifier: snap.modifier,
    }
}

fn solve_unmatched(
    src: &[(u16, u16, CellSnapshot)],
    dst: &[(u16, u16, CellSnapshot)],
    weights: &MorphWeights,
) -> (Vec<DisplacedCell>, Vec<OrphanCell>, Vec<OrphanCell>) {
    if src.is_empty() && dst.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    if src.is_empty() {
        let appearing = dst.iter().map(|(x, y, s)| orphan_from(*x, *y, s)).collect();
        return (Vec::new(), appearing, Vec::new());
    }

    if dst.is_empty() {
        let disappearing = src.iter().map(|(x, y, s)| orphan_from(*x, *y, s)).collect();
        return (Vec::new(), Vec::new(), disappearing);
    }

    let n = src.len();
    let m = dst.len();
    let mut cost = vec![vec![0.0f32; m]; n];

    for (i, (sx, sy, ss)) in src.iter().enumerate() {
        for (j, (dx, dy, ds)) in dst.iter().enumerate() {
            cost[i][j] = cell_cost(*sx, *sy, ss, *dx, *dy, ds, weights);
        }
    }

    // Above this cost, fade out + fade in is cheaper than displacement.
    let threshold = weights.glyph_mismatch * weights.glyph * 2.0
        + weights.spatial * 100.0
        + weights.color * 0.5;

    let assignment = hungarian(&cost, n, m);

    let mut displaced = Vec::new();
    let mut appearing = Vec::new();
    let mut disappearing = Vec::new();
    let mut dst_matched = vec![false; m];

    for (i, matched_j) in assignment.iter().enumerate() {
        match matched_j {
            Some(j) if cost[i][*j] <= threshold => {
                let (sx, sy, ss) = &src[i];
                let (dx, dy, ds) = &dst[*j];

                displaced.push(DisplacedCell {
                    src_x: *sx,
                    src_y: *sy,
                    dst_x: *dx,
                    dst_y: *dy,
                    src_symbol: ss.symbol.clone(),
                    dst_symbol: ds.symbol.clone(),
                    src_fg: ss.fg,
                    dst_fg: ds.fg,
                    src_bg: ss.bg,
                    dst_bg: ds.bg,
                    src_modifier: ss.modifier,
                    dst_modifier: ds.modifier,
                });

                dst_matched[*j] = true;
            }

            _ => {
                let (x, y, snap) = &src[i];
                disappearing.push(orphan_from(*x, *y, snap));
            }
        }
    }

    for (j, (x, y, snap)) in dst.iter().enumerate() {
        if !dst_matched[j] {
            appearing.push(orphan_from(*x, *y, snap));
        }
    }

    (displaced, appearing, disappearing)
}

fn cell_cost(
    sx: u16,
    sy: u16,
    ss: &CellSnapshot,
    dx: u16,
    dy: u16,
    ds: &CellSnapshot,
    w: &MorphWeights,
) -> f32 {
    let spatial = {
        let dx_f = (dx as f32) - (sx as f32);
        let dy_f = (dy as f32) - (sy as f32);
        dx_f * dx_f + dy_f * dy_f
    };

    let glyph = if ss.symbol == ds.symbol {
        0.0
    } else {
        w.glyph_mismatch
    };

    let color = match (ss.fg.oklch, ds.fg.oklch) {
        (Some(a), Some(b)) => oklch::distance(a, b),
        _ => 0.5,
    };

    w.spatial * spatial + w.glyph * glyph + w.color * color
}

/// Pads to square internally — the algorithm requires it.
fn hungarian(cost: &[Vec<f32>], n: usize, m: usize) -> Vec<Option<usize>> {
    let size = n.max(m);
    let mut c = vec![vec![0.0f32; size]; size];

    for (i, row) in cost.iter().enumerate() {
        for (j, &val) in row.iter().enumerate() {
            c[i][j] = val;
        }
    }

    let mut u = vec![0.0f32; size + 1];
    let mut v = vec![0.0f32; size + 1];
    let mut assignment = vec![0usize; size + 1];
    let mut way = vec![0usize; size + 1];

    for i in 1..=size {
        assignment[0] = i;
        let mut j0 = 0usize;
        let mut min_v = vec![f32::INFINITY; size + 1];
        let mut used = vec![false; size + 1];

        loop {
            used[j0] = true;
            let i0 = assignment[j0];
            let mut delta = f32::INFINITY;
            let mut j1 = 0usize;

            for j in 1..=size {
                if used[j] {
                    continue;
                }

                let cur = c[i0 - 1][j - 1] - u[i0] - v[j];

                if cur < min_v[j] {
                    min_v[j] = cur;
                    way[j] = j0;
                }

                if min_v[j] < delta {
                    delta = min_v[j];
                    j1 = j;
                }
            }

            for j in 0..=size {
                if used[j] {
                    u[assignment[j]] += delta;
                    v[j] -= delta;
                } else {
                    min_v[j] -= delta;
                }
            }

            j0 = j1;

            if assignment[j0] == 0 {
                break;
            }
        }

        loop {
            let prev = way[j0];
            assignment[j0] = assignment[prev];
            j0 = prev;

            if j0 == 0 {
                break;
            }
        }
    }

    // Filter to real (non-padded) assignments.
    let mut result = vec![None; n];

    for j in 1..=size {
        let i = assignment[j];

        if i >= 1 && i <= n && j >= 1 && j <= m {
            result[i - 1] = Some(j - 1);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Style};

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
    fn identical_buffers_all_stable() {
        let a = make_buffer(3, 1, &[((0, 0), "A", Color::Red), ((1, 0), "B", Color::Blue)]);
        let b = make_buffer(3, 1, &[((0, 0), "A", Color::Red), ((1, 0), "B", Color::Blue)]);
        let plan = diff(&a, &b, &MorphWeights::LIQUID);

        assert_eq!(plan.stable.len(), 3);
        assert!(plan.mutating.is_empty());
        assert!(plan.displaced.is_empty());
        assert!(plan.appearing.is_empty());
        assert!(plan.disappearing.is_empty());
    }

    #[test]
    fn color_change_is_mutating() {
        let a = make_buffer(1, 1, &[((0, 0), "X", Color::Red)]);
        let b = make_buffer(1, 1, &[((0, 0), "X", Color::Blue)]);
        let plan = diff(&a, &b, &MorphWeights::LIQUID);

        assert!(plan.stable.is_empty());
        assert_eq!(plan.mutating.len(), 1);
        assert_eq!(plan.mutating[0].src_symbol, "X");
    }

    #[test]
    fn appearing_cell() {
        let a = make_buffer(2, 1, &[]);
        let b = make_buffer(2, 1, &[((1, 0), "Z", Color::Green)]);
        let plan = diff(&a, &b, &MorphWeights::LIQUID);

        assert_eq!(plan.appearing.len(), 1);
        assert_eq!(plan.appearing[0].symbol, "Z");
    }

    #[test]
    fn disappearing_cell() {
        let a = make_buffer(2, 1, &[((0, 0), "Z", Color::Green)]);
        let b = make_buffer(2, 1, &[]);
        let plan = diff(&a, &b, &MorphWeights::LIQUID);

        assert_eq!(plan.disappearing.len(), 1);
        assert_eq!(plan.disappearing[0].symbol, "Z");
    }

    #[test]
    fn displaced_cell() {
        let a = make_buffer(3, 1, &[((0, 0), "M", Color::Red)]);
        let b = make_buffer(3, 1, &[((2, 0), "M", Color::Red)]);
        let plan = diff(&a, &b, &MorphWeights::LIQUID);

        assert_eq!(plan.displaced.len(), 1);
        assert_eq!(plan.displaced[0].src_x, 0);
        assert_eq!(plan.displaced[0].dst_x, 2);
    }

    #[test]
    fn hungarian_identity() {
        let cost = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let result = hungarian(&cost, 2, 2);

        assert_eq!(result, vec![Some(0), Some(1)]);
    }

    #[test]
    fn hungarian_swap() {
        let cost = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let result = hungarian(&cost, 2, 2);

        assert_eq!(result, vec![Some(1), Some(0)]);
    }

    #[test]
    fn hungarian_rectangular() {
        let cost = vec![vec![10.0, 1.0], vec![1.0, 10.0], vec![5.0, 5.0]];
        let result = hungarian(&cost, 3, 2);

        assert_eq!(result[0], Some(1));
        assert_eq!(result[1], Some(0));
        assert_eq!(result[2], None);
    }
}
