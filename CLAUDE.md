# tui-morph

Frame-level morphing layer for ratatui. Interpolates between discrete terminal frames using Oklch color space, producing smooth visual transitions with zero application-level awareness.

## Project Structure

```
Cargo.toml              # workspace root
CLAUDE.md               # you are here
tui-morph/          # core library crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs       # public API surface
        ├── solver.rs    # frame diffing and cell correspondence (Hungarian algorithm)
        ├── interpolate.rs # per-cell interpolation (glyph, Oklch color, position)
        ├── plan.rs      # InterpolationPlan: frozen diff artifact, pure data
        ├── weights.rs   # MorphWeights and named presets
        ├── oklch.rs     # Oklch conversion and perceptual interpolation
        ├── easing.rs    # easing functions (linear, ease-in-out, custom curves)
        └── backend.rs   # MorphBackend: wraps any ratatui Backend
tui-morph-harness/   # test harness / visual demo crate
    ├── Cargo.toml
    └── src/
        └── main.rs      # interactive demo for testing morph effects
```

## Architecture

### Core Invariant

The morph layer is **stateless between logical frames**. An `InterpolationPlan` is a pure function artifact produced from two buffers. Rendering at any `t ∈ [0.0, 1.0]` is a pure function `(plan, t) → buffer`. Interpolated frames are ephemeral and never fed back into the solver.

### Data Flow

```
App calls render() → MorphBackend intercepts Buffer
  → solver::diff(prev_logical, next_logical) → InterpolationPlan
  → tick loop: interpolate(plan, t) → ephemeral buffer → backend.flush()
  → on completion: next_logical becomes prev_logical
```

### Interrupted Transitions

When a new logical frame arrives mid-transition, snapshot the current interpolated buffer and use it as the source for the new plan. Error is bounded to one transition's worth of drift and converges to the correct target.

### Cell Classification

The solver partitions cells into four categories:

1. **Stable** — same position, same glyph, same style. Zero cost.
2. **Mutating** — same position, different content. Interpolate in-place via Oklch.
3. **Displaced** — content exists in both frames at different positions. Solve assignment via cost matrix.
4. **Orphan** — exists in only one frame. Fade in/out by driving Oklch lightness toward/from zero.

### Cost Function

This is just a suggestion.

```rust
fn cell_cost(src: &Cell, dst: &Cell, w: &MorphWeights) -> f32 {
    let spatial = squared_euclidean(src.pos, dst.pos) as f32;
    let glyph = if src.glyph == dst.glyph { 0.0 } else { w.glyph_mismatch };
    let color = oklch_distance(&src.style, &dst.style);
    w.spatial * spatial + w.glyph * glyph + w.color * color
}
```

Squared euclidean for spatial cost — penalizes long moves quadratically, naturally preferring many short moves over few long ones.

### Weight Presets

Weights are the style. Different weight profiles produce completely different visual personalities from the same solver:

- `LIQUID` — high spatial weight, everything flows to destination
- `CRISP` — high glyph weight, in-place rewrites, text mutates
- `FADE` — high color weight, whole-frame Oklch crossfade

### Oklch Interpolation

All color interpolation happens in Oklch space for perceptual linearity.

- **L** (lightness): drives fade-in/fade-out for orphan cells
- **C** (chroma): lerp
- **h** (hue): circular lerp via shortest arc
- Glyph snap occurs when lightness crosses below a legibility threshold — the old glyph dissolves into darkness, the new one emerges. Crossfade through black hides the discrete glyph discontinuity.

### Ratatui Integration

`MorphBackend<B: Backend>` wraps any ratatui backend. The application renders normally via `Terminal::draw()`. The wrapper intercepts buffers, manages the interpolation tick loop, and flushes interpolated frames to the inner backend. The app has zero awareness of morphing.

## Conventions

- Nix all the things, make a flake.nix using cranelib and rust-overlay
- Rust 2024 edition, latest stable toolchain
- `cargo fmt` and `cargo clippy` clean before committing
- No `unsafe` without a comment justifying it
- No `unwrap()` in library code — use `Result` or `Option` combinators
- Prefer `f32` for all interpolation math (terminal cells don't need f64 precision)
- The library crate (`tui-morph`) must have zero side effects — no printing, no I/O, no global state
- The harness crate owns all terminal I/O, event handling, and demo logic

## Dependencies

### tui-morph (library)
- `ratatui` — buffer types, Backend trait, Style/Color
- `palette` or hand-rolled Oklch — evaluate which is lighter; hand-rolling sRGB↔Oklch is ~40 lines and avoids a dep tree. Start hand-rolled, graduate to `palette` only if needed.

### tui-morph-harness (demo)

- `ratatui` + `crossterm` — terminal backend
- `tui-morph` — path dep to workspace sibling

## Implementation Notes

### Hungarian Algorithm
For the displaced-cell assignment problem. O(n³) but n is typically 50–200 displaced cells per frame transition, so it's trivial. Consider the `pathfinding` crate or a standalone implementation. Keep it in `solver.rs`.

### Tick Scheduling
The interpolation tick loop must coexist with the application's event loop. Options:
- Simplest: the `MorphBackend` runs ticks synchronously during `flush()`, blocking for the transition duration. Acceptable for short transitions (150–300ms).
- Better: expose a `MorphBackend::tick(&mut self, elapsed: Duration)` that the app can call from its own event loop. Non-blocking, app retains control.
- Evaluate which to implement first. The blocking approach is a correct MVP.

### sRGB ↔ Oklch
ratatui's `Color::Rgb(r, g, b)` is sRGB. Conversion path: sRGB → linear RGB → Oklab → Oklch. Reference: https://bottosson.github.io/posts/oklab/

### Legibility Threshold
The lightness value below which a glyph is considered invisible against a dark background. Start with `L = 0.15` and tune empirically in the harness. This threshold determines when the glyph snap occurs during crossfade.

## Testing Strategy

- Unit tests in the library for solver correctness (known input pairs → expected correspondence)
- Unit tests for Oklch round-trip accuracy (sRGB → Oklch → sRGB within ±1 per channel)
- Property tests: `tick(plan, 0.0) == source`, `tick(plan, 1.0) == target`
- The harness is the primary integration test — visual inspection of morph quality
- Consider recording frame sequences to disk for regression testing (serialize Buffer snapshots)

## Open Questions

- Should the solver attempt SVD decomposition of the displacement field to extract principal motion components (bulk translation, rotation, scaling)? This would let easing respect structural coherence — bulk motion moves linearly while residuals follow a different curve. Possibly a v2 feature.
- Density-based glyph interpolation (mapping characters by visual ink density for dissolve effects) — complementary to Oklch lightness but adds another axis. Park for now.
- Async solve with double-buffered plans for large diffs — premature but architecturally sound for later.
