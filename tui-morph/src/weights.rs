/// Controls the relative influence of spatial, glyph, and color terms
/// in the solver's cost function.
#[derive(Debug, Clone, Copy)]
pub struct MorphWeights {
    pub spatial: f32,
    pub glyph: f32,
    pub color: f32,
    pub glyph_mismatch: f32,
}

impl MorphWeights {
    /// High spatial weight — cells flow to their destination positions.
    pub const LIQUID: Self = Self {
        spatial: 1.0,
        glyph: 0.1,
        color: 0.2,
        glyph_mismatch: 5.0,
    };

    /// High glyph weight — in-place rewrites, text mutates rather than moves.
    pub const CRISP: Self = Self {
        spatial: 0.2,
        glyph: 1.0,
        color: 0.3,
        glyph_mismatch: 20.0,
    };

    /// High color weight — whole-frame Oklch crossfade.
    pub const FADE: Self = Self {
        spatial: 0.1,
        glyph: 0.1,
        color: 1.0,
        glyph_mismatch: 2.0,
    };
}
