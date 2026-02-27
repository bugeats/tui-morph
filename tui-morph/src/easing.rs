pub fn linear(t: f32) -> f32 {
    t
}

pub fn ease_in(t: f32) -> f32 {
    t * t
}

pub fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

/// CSS `cubic-bezier(x1, y1, x2, y2)` semantics.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> impl Fn(f32) -> f32 {
    move |t| {
        let bezier_t = solve_bezier_t(t, x1, x2);
        sample_bezier(bezier_t, y1, y2)
    }
}

fn solve_bezier_t(x: f32, x1: f32, x2: f32) -> f32 {
    let mut t = x;

    for _ in 0..8 {
        let residual = sample_bezier(t, x1, x2) - x;

        if residual.abs() < 1e-6 {
            return t;
        }

        let slope = bezier_derivative(t, x1, x2);

        if slope.abs() < 1e-6 {
            break;
        }

        t -= residual / slope;
    }

    t
}

/// Evaluate cubic bezier with endpoints (0,0) and (1,1).
fn sample_bezier(t: f32, p1: f32, p2: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

fn bezier_derivative(t: f32, p1: f32, p2: f32) -> f32 {
    let mt = 1.0 - t;

    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_boundaries(f: impl Fn(f32) -> f32) {
        assert!(f(0.0).abs() < 1e-6, "f(0) = {}, expected 0", f(0.0));
        assert!((f(1.0) - 1.0).abs() < 1e-6, "f(1) = {}, expected 1", f(1.0));
    }

    fn assert_monotonic(f: impl Fn(f32) -> f32) {
        let mut prev = f(0.0);

        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let val = f(t);
            assert!(val >= prev - 1e-6, "non-monotonic at t={t}: {prev} > {val}");
            prev = val;
        }
    }

    #[test]
    fn linear_is_identity() {
        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((linear(t) - t).abs() < 1e-6);
        }
    }

    #[test]
    fn builtin_boundaries() {
        assert_boundaries(ease_in);
        assert_boundaries(ease_out);
        assert_boundaries(ease_in_out);
    }

    #[test]
    fn builtin_monotonic() {
        assert_monotonic(ease_in);
        assert_monotonic(ease_out);
        assert_monotonic(ease_in_out);
    }

    #[test]
    fn ease_in_starts_slow() {
        assert!(ease_in(0.25) < 0.25);
    }

    #[test]
    fn ease_out_starts_fast() {
        assert!(ease_out(0.25) > 0.25);
    }

    #[test]
    fn ease_in_out_symmetric() {
        assert!((ease_in_out(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn cubic_bezier_boundaries() {
        let ease = cubic_bezier(0.25, 0.1, 0.25, 1.0);
        assert_boundaries(ease);
    }

    #[test]
    fn cubic_bezier_linear() {
        let ease = cubic_bezier(0.0, 0.0, 1.0, 1.0);

        for i in 0..=10 {
            let t = i as f32 / 10.0;
            assert!((ease(t) - t).abs() < 0.01, "at t={t}: {}", ease(t));
        }
    }
}
