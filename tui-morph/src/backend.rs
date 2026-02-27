use std::io;
use std::time::{Duration, Instant};

use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect, Size};
use ratatui::Terminal;

use crate::easing;
use crate::interpolate;
use crate::solver;
use crate::weights::MorphWeights;

pub struct MorphConfig {
    pub weights: MorphWeights,
    pub duration: Duration,
    pub easing: fn(f32) -> f32,
    pub fps: u32,
}

impl Default for MorphConfig {
    fn default() -> Self {
        Self {
            weights: MorphWeights::LIQUID,
            duration: Duration::from_millis(200),
            easing: easing::ease_in_out,
            fps: 60,
        }
    }
}

/// Wraps any ratatui Backend, intercepting frames to produce smooth morph transitions.
///
/// Use `MorphBackend::wrap` to create a `Terminal<MorphBackend<B>>`.
/// The application renders normally — morphing is transparent.
pub struct MorphBackend<B: Backend> {
    inner: B,
    config: MorphConfig,

    /// Full current frame, assembled incrementally from Terminal's deltas.
    current_frame: Buffer,

    /// Previous logical frame for diffing. `None` on first render.
    prev_frame: Option<Buffer>,

    /// Last frame sent to inner backend, for efficient diff-based updates.
    last_flushed: Buffer,
}

impl<B: Backend> MorphBackend<B> {
    pub fn new(inner: B, config: MorphConfig) -> io::Result<Self> {
        let size = inner.size()?;
        let area = Rect::new(0, 0, size.width, size.height);
        let empty = Buffer::empty(area);

        Ok(Self {
            inner,
            config,
            current_frame: empty.clone(),
            prev_frame: None,
            last_flushed: empty,
        })
    }

    pub fn wrap(backend: B, config: MorphConfig) -> io::Result<Terminal<Self>> {
        let morph = Self::new(backend, config)?;
        Terminal::new(morph)
    }

    fn flush_buffer_to_inner(&mut self, buf: &Buffer) -> io::Result<()> {
        let updates = self.last_flushed.diff(buf);
        self.inner.draw(updates.into_iter())?;
        self.inner.flush()?;
        self.last_flushed = buf.clone();
        Ok(())
    }

    fn run_transition(&mut self, prev: &Buffer, next: &Buffer) -> io::Result<()> {
        let plan = solver::diff(prev, next, &self.config.weights);
        let frame_interval = Duration::from_secs(1) / self.config.fps;
        let start = Instant::now();

        loop {
            let elapsed = start.elapsed();
            let raw_t = (elapsed.as_secs_f32() / self.config.duration.as_secs_f32()).min(1.0);
            let t = (self.config.easing)(raw_t);

            let interpolated = interpolate::render(&plan, t);
            self.flush_buffer_to_inner(&interpolated)?;

            if raw_t >= 1.0 {
                break;
            }

            let next_tick = start + frame_interval * (elapsed.as_secs_f32() / frame_interval.as_secs_f32()).ceil() as u32;
            let now = Instant::now();

            if next_tick > now {
                std::thread::sleep(next_tick - now);
            }
        }

        Ok(())
    }
}

impl<B: Backend> Backend for MorphBackend<B> {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a ratatui::buffer::Cell)>,
    {
        for (x, y, cell) in content {
            if x < self.current_frame.area().width && y < self.current_frame.area().height {
                self.current_frame[(x, y)] = cell.clone();
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        let next = self.current_frame.clone();

        if let Some(prev) = self.prev_frame.take() {
            self.run_transition(&prev, &next)?;
        } else {
            self.flush_buffer_to_inner(&next)?;
        }

        self.prev_frame = Some(next);

        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.inner.hide_cursor()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.inner.show_cursor()
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        self.inner.get_cursor_position()
    }

    fn set_cursor_position<P: Into<ratatui::layout::Position>>(
        &mut self,
        position: P,
    ) -> io::Result<()> {
        self.inner.set_cursor_position(position)
    }

    fn clear(&mut self) -> io::Result<()> {
        self.inner.clear()
    }

    fn clear_region(&mut self, clear_type: ratatui::backend::ClearType) -> io::Result<()> {
        self.inner.clear_region(clear_type)
    }

    fn size(&self) -> io::Result<Size> {
        self.inner.size()
    }

    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        self.inner.window_size()
    }
}
