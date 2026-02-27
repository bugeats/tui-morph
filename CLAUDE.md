# tui-morph

Frame-level morphing layer for ratatui. Interpolates between discrete terminal frames using Oklch color space, producing smooth visual transitions with zero application-level awareness.

## Project Structure

```
Cargo.toml              # virtual workspace root
tui-morph/              # core library crate (zero side effects)
    └── src/
        ├── lib.rs          # public API surface
        ├── oklch.rs        # sRGB↔Oklch conversion, perceptual lerp
        ├── easing.rs       # easing functions, cubic bezier
        ├── weights.rs      # MorphWeights presets (LIQUID, CRISP, FADE)
        ├── plan.rs         # InterpolationPlan: frozen diff artifact
        ├── solver.rs       # frame diffing, Hungarian assignment
        ├── interpolate.rs  # per-cell interpolation (glyph, color, position)
        └── backend.rs      # MorphBackend<B>: wraps any ratatui Backend
tui-morph-harness/      # visual demo (owns all terminal I/O)
    └── src/main.rs
```

See [docs/architecture.md](docs/architecture.md) for full design spec.

## Conventions

- Nix build via cranelib + rust-overlay (`flake.nix`)
- Rust 2024 edition, latest stable toolchain
- `cargo fmt` and `cargo clippy` clean before committing
- No `unsafe` without justification
- No `unwrap()` in library code
- `f32` for all interpolation math
- Hand-rolled sRGB↔Oklch (no `palette` dep)

## Dependencies

- `ratatui` 0.29 — buffer types, Backend trait, Style/Color
- `crossterm` 0.28 — terminal backend (harness only)

## Testing Strategy

- Unit tests: Oklch round-trip (±1/channel), solver correctness, boundary properties
- Property tests: `interpolate(plan, 0.0) == source`, `interpolate(plan, 1.0) == target`
- Harness: visual integration testing

## Current Focus

All 9 implementation arcs complete (29 tests). Harness: 6-scene interactive demo (`nix run`). Visual testing in progress.
