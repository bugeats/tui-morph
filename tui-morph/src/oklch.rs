/// Oklch color: perceptually uniform lightness, chroma, and hue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

#[derive(Debug, Clone, Copy)]
struct Oklab {
    l: f32,
    a: f32,
    b: f32,
}

#[derive(Debug, Clone, Copy)]
struct LinRgb {
    r: f32,
    g: f32,
    b: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

fn linear_rgb_to_oklab(rgb: LinRgb) -> Oklab {
    let LinRgb { r, g, b } = rgb;

    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l = l.cbrt();
    let m = m.cbrt();
    let s = s.cbrt();

    Oklab {
        l: 0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        a: 1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        b: 0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    }
}

fn oklab_to_linear_rgb(lab: Oklab) -> LinRgb {
    let Oklab { l, a, b } = lab;

    let l_ = l + 0.3963377774 * a + 0.2158037573 * b;
    let m_ = l - 0.1055613458 * a - 0.0638541728 * b;
    let s_ = l - 0.0894841775 * a - 1.2914855480 * b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    LinRgb {
        r: 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        g: -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        b: -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    }
}

fn oklab_to_oklch(lab: Oklab) -> Oklch {
    let c = (lab.a * lab.a + lab.b * lab.b).sqrt();

    let h = if c < 1e-8 { 0.0 } else { lab.b.atan2(lab.a) };

    Oklch { l: lab.l, c, h }
}

fn oklch_to_oklab(lch: Oklch) -> Oklab {
    Oklab {
        l: lch.l,
        a: lch.c * lch.h.cos(),
        b: lch.c * lch.h.sin(),
    }
}

pub fn srgb_to_oklch(r: u8, g: u8, b: u8) -> Oklch {
    let lin = LinRgb {
        r: srgb_to_linear(r as f32 / 255.0),
        g: srgb_to_linear(g as f32 / 255.0),
        b: srgb_to_linear(b as f32 / 255.0),
    };

    oklab_to_oklch(linear_rgb_to_oklab(lin))
}

pub fn oklch_to_srgb(lch: Oklch) -> (u8, u8, u8) {
    let lin = oklab_to_linear_rgb(oklch_to_oklab(lch));

    let to_u8 = |c: f32| (linear_to_srgb(c.clamp(0.0, 1.0)) * 255.0 + 0.5) as u8;

    (to_u8(lin.r), to_u8(lin.g), to_u8(lin.b))
}

/// Hue interpolates via shortest arc.
pub fn lerp(a: Oklch, b: Oklch, t: f32) -> Oklch {
    use std::f32::consts::PI;

    let l = a.l + (b.l - a.l) * t;
    let c = a.c + (b.c - a.c) * t;

    let mut dh = b.h - a.h;

    if dh > PI {
        dh -= 2.0 * PI;
    } else if dh < -PI {
        dh += 2.0 * PI;
    }

    let h = a.h + dh * t;

    Oklch { l, c, h }
}

/// Convert a ratatui Color to Oklch, if it has a concrete RGB representation.
pub fn from_color(color: ratatui::style::Color) -> Option<Oklch> {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => Some(srgb_to_oklch(r, g, b)),
        Color::Black => Some(srgb_to_oklch(0, 0, 0)),
        Color::Red => Some(srgb_to_oklch(128, 0, 0)),
        Color::Green => Some(srgb_to_oklch(0, 128, 0)),
        Color::Yellow => Some(srgb_to_oklch(128, 128, 0)),
        Color::Blue => Some(srgb_to_oklch(0, 0, 128)),
        Color::Magenta => Some(srgb_to_oklch(128, 0, 128)),
        Color::Cyan => Some(srgb_to_oklch(0, 128, 128)),
        Color::Gray => Some(srgb_to_oklch(192, 192, 192)),
        Color::DarkGray => Some(srgb_to_oklch(128, 128, 128)),
        Color::LightRed => Some(srgb_to_oklch(255, 0, 0)),
        Color::LightGreen => Some(srgb_to_oklch(0, 255, 0)),
        Color::LightYellow => Some(srgb_to_oklch(255, 255, 0)),
        Color::LightBlue => Some(srgb_to_oklch(0, 0, 255)),
        Color::LightMagenta => Some(srgb_to_oklch(255, 0, 255)),
        Color::LightCyan => Some(srgb_to_oklch(0, 255, 255)),
        Color::White => Some(srgb_to_oklch(255, 255, 255)),
        Color::Reset | Color::Indexed(_) => None,
    }
}

pub fn to_color(lch: Oklch) -> ratatui::style::Color {
    let (r, g, b) = oklch_to_srgb(lch);
    ratatui::style::Color::Rgb(r, g, b)
}

/// Perceptual distance between two Oklch colors.
pub fn distance(a: Oklch, b: Oklch) -> f32 {
    let lab_a = oklch_to_oklab(a);
    let lab_b = oklch_to_oklab(b);

    let dl = lab_a.l - lab_b.l;
    let da = lab_a.a - lab_b.a;
    let db = lab_a.b - lab_b.b;

    (dl * dl + da * da + db * db).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round-trip sRGB → Oklch → sRGB must be within ±1 per channel.
    fn assert_round_trip(r: u8, g: u8, b: u8) {
        let lch = srgb_to_oklch(r, g, b);
        let (r2, g2, b2) = oklch_to_srgb(lch);

        assert!(
            (r as i16 - r2 as i16).unsigned_abs() <= 1
                && (g as i16 - g2 as i16).unsigned_abs() <= 1
                && (b as i16 - b2 as i16).unsigned_abs() <= 1,
            "round-trip failed: ({r}, {g}, {b}) → {lch:?} → ({r2}, {g2}, {b2})"
        );
    }

    #[test]
    fn round_trip_primaries() {
        assert_round_trip(255, 0, 0);
        assert_round_trip(0, 255, 0);
        assert_round_trip(0, 0, 255);
    }

    #[test]
    fn round_trip_grays() {
        for v in (0..=255).step_by(17) {
            assert_round_trip(v, v, v);
        }
    }

    #[test]
    fn round_trip_assorted() {
        let samples = [
            (128, 64, 32),
            (10, 200, 150),
            (255, 128, 0),
            (100, 100, 100),
            (1, 1, 1),
            (254, 254, 254),
        ];

        for (r, g, b) in samples {
            assert_round_trip(r, g, b);
        }
    }

    #[test]
    fn black_has_zero_lightness() {
        let lch = srgb_to_oklch(0, 0, 0);
        assert!(lch.l.abs() < 1e-6);
    }

    #[test]
    fn white_has_unit_lightness() {
        let lch = srgb_to_oklch(255, 255, 255);
        assert!((lch.l - 1.0).abs() < 0.01);
    }

    #[test]
    fn grays_have_zero_chroma() {
        for v in (0..=255).step_by(51) {
            let lch = srgb_to_oklch(v, v, v);
            assert!(lch.c < 1e-4, "gray {v} had chroma {}", lch.c);
        }
    }

    #[test]
    fn lerp_endpoints() {
        let a = srgb_to_oklch(255, 0, 0);
        let b = srgb_to_oklch(0, 0, 255);

        let at_zero = lerp(a, b, 0.0);
        let at_one = lerp(a, b, 1.0);

        assert!((at_zero.l - a.l).abs() < 1e-6);
        assert!((at_zero.c - a.c).abs() < 1e-6);
        assert!((at_one.l - b.l).abs() < 1e-6);
        assert!((at_one.c - b.c).abs() < 1e-6);
    }

    #[test]
    fn distance_is_zero_for_same_color() {
        let c = srgb_to_oklch(100, 150, 200);
        assert!(distance(c, c) < 1e-6);
    }

    #[test]
    fn distance_is_symmetric() {
        let a = srgb_to_oklch(255, 0, 0);
        let b = srgb_to_oklch(0, 255, 0);

        assert!((distance(a, b) - distance(b, a)).abs() < 1e-6);
    }
}
