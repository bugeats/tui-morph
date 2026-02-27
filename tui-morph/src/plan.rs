use ratatui::style::{Color, Modifier};

use crate::oklch::Oklch;

/// Frozen diff artifact between two frames. Produced by the solver, consumed by the interpolator.
pub struct InterpolationPlan {
    pub width: u16,
    pub height: u16,
    pub stable: Vec<StableCell>,
    pub mutating: Vec<MutatingCell>,
    pub displaced: Vec<DisplacedCell>,
    pub appearing: Vec<OrphanCell>,
    pub disappearing: Vec<OrphanCell>,
}

pub struct StableCell {
    pub x: u16,
    pub y: u16,
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

pub struct MutatingCell {
    pub x: u16,
    pub y: u16,
    pub src_symbol: String,
    pub dst_symbol: String,
    pub src_fg: ColorPair,
    pub dst_fg: ColorPair,
    pub src_bg: ColorPair,
    pub dst_bg: ColorPair,
    pub src_modifier: Modifier,
    pub dst_modifier: Modifier,
}

pub struct DisplacedCell {
    pub src_x: u16,
    pub src_y: u16,
    pub dst_x: u16,
    pub dst_y: u16,
    pub src_symbol: String,
    pub dst_symbol: String,
    pub src_fg: ColorPair,
    pub dst_fg: ColorPair,
    pub src_bg: ColorPair,
    pub dst_bg: ColorPair,
    pub src_modifier: Modifier,
    pub dst_modifier: Modifier,
}

/// Exists in only one frame. Fades in (appearing) or out (disappearing).
///
/// `counter_bg`: the background at this position in the *other* frame,
/// so orphan bg can interpolate toward it instead of fading to black.
pub struct OrphanCell {
    pub x: u16,
    pub y: u16,
    pub symbol: String,
    pub fg: ColorPair,
    pub bg: ColorPair,
    pub counter_bg: ColorPair,
    pub modifier: Modifier,
}

/// `None` oklch means the color can't be interpolated (Reset, Indexed).
#[derive(Clone, Copy)]
pub struct ColorPair {
    pub raw: Color,
    pub oklch: Option<Oklch>,
}

impl ColorPair {
    pub fn from_color(color: Color) -> Self {
        Self {
            raw: color,
            oklch: crate::oklch::from_color(color),
        }
    }
}
