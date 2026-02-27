# Architecture

## Core Invariant

The morph layer is **stateless between logical frames**. An `InterpolationPlan` is a pure function artifact produced from two buffers. Rendering at any `t ∈ [0.0, 1.0]` is a pure function `(plan, t) → buffer`. Interpolated frames are ephemeral and never fed back into the solver.

## Data Flow

```
App calls render() → MorphBackend intercepts Buffer
  → solver::diff(prev_logical, next_logical) → InterpolationPlan
  → tick loop: interpolate(plan, t) → ephemeral buffer → backend.flush()
  → on completion: next_logical becomes prev_logical
```

## Interrupted Transitions (future)

Not yet implemented. The current blocking tick loop prevents mid-transition interruption. When non-blocking mode is added: snapshot the current interpolated buffer and use it as the source for the new plan. Error is bounded to one transition's worth of drift and converges to the correct target.

## Cell Classification

The solver partitions cells into four categories:

1. **Stable** — same position, same glyph, same style. Zero cost.
2. **Mutating** — same position, different content. Interpolate in-place via Oklch.
3. **Displaced** — content exists in both frames at different positions. Solve assignment via cost matrix.
4. **Orphan** — exists in only one frame. Fade in/out by driving Oklch lightness toward/from zero.

## Cost Function

```rust
fn cell_cost(src: &Cell, dst: &Cell, w: &MorphWeights) -> f32 {
    let spatial = squared_euclidean(src.pos, dst.pos) as f32;
    let glyph = if src.glyph == dst.glyph { 0.0 } else { w.glyph_mismatch };
    let color = oklch_distance(&src.style, &dst.style);
    w.spatial * spatial + w.glyph * glyph + w.color * color
}
```

Squared euclidean for spatial cost — penalizes long moves quadratically, naturally preferring many short moves over few long ones.

## Weight Presets

Weights are the style. Different weight profiles produce completely different visual personalities from the same solver:

- `LIQUID` — high spatial weight, everything flows to destination
- `CRISP` — high glyph weight, in-place rewrites, text mutates
- `FADE` — high color weight, whole-frame Oklch crossfade

## Oklch Interpolation

All color interpolation happens in Oklch space for perceptual linearity.

- **L** (lightness): drives fade-in/fade-out for orphan cells
- **C** (chroma): lerp
- **h** (hue): circular lerp via shortest arc
- Glyph snap occurs when lightness crosses below a legibility threshold — the old glyph dissolves into darkness, the new one emerges. Crossfade through black hides the discrete glyph discontinuity.

## Ratatui Integration

`MorphBackend<B: Backend>` wraps any ratatui backend. The application renders normally via `Terminal::draw()`. The wrapper intercepts buffers, manages the interpolation tick loop, and flushes interpolated frames to the inner backend. The app has zero awareness of morphing.

## Implementation Notes

### Hungarian Algorithm

For the displaced-cell assignment problem. O(n³) but n is typically 50–200 displaced cells per frame transition, so it's trivial. Standalone implementation in `solver.rs`.

### Tick Scheduling

MVP: blocking tick loop during `flush()`. Acceptable for short transitions (150–300ms). Future: expose `MorphBackend::tick(&mut self, elapsed: Duration)` for app-driven non-blocking control.

### sRGB ↔ Oklch

ratatui's `Color::Rgb(r, g, b)` is sRGB. Conversion path: sRGB → linear RGB → Oklab → Oklch. Reference: https://bottosson.github.io/posts/oklab/

### Legibility Threshold

Lightness value below which a glyph is invisible against a dark background. Start with `L = 0.15`, tune empirically.

## Open Questions

- SVD decomposition of displacement field for structural coherence in easing — v2.
- Density-based glyph interpolation (ink density for dissolve effects) — parked.
- Async solve with double-buffered plans for large diffs — parked.
