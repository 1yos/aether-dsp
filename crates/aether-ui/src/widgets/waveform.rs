//! Waveform display widget — GPU-rendered.
//!
//! Renders an audio waveform as a filled polygon directly on the GPU.
//! Handles millions of samples at 60fps by downsampling to screen resolution.

use gpui::*;
use crate::theme::Theme;

pub struct Waveform {
    /// Downsampled peak data: (min, max) per pixel column.
    pub peaks: Vec<(f32, f32)>,
    pub color: Rgba,
    pub width: f32,
    pub height: f32,
}

impl Waveform {
    pub fn new(samples: &[f32], width: f32, height: f32, color: Rgba) -> Self {
        let peaks = compute_peaks(samples, width as usize);
        Self { peaks, color, width, height }
    }

    pub fn empty(width: f32, height: f32) -> Self {
        Self {
            peaks: Vec::new(),
            color: Theme::ACCENT,
            width,
            height,
        }
    }
}

fn compute_peaks(samples: &[f32], columns: usize) -> Vec<(f32, f32)> {
    if samples.is_empty() || columns == 0 {
        return Vec::new();
    }
    let samples_per_col = (samples.len() as f32 / columns as f32).max(1.0);
    (0..columns)
        .map(|col| {
            let start = (col as f32 * samples_per_col) as usize;
            let end = ((col + 1) as f32 * samples_per_col) as usize;
            let end = end.min(samples.len());
            let slice = &samples[start..end];
            if slice.is_empty() {
                return (0.0, 0.0);
            }
            let min = slice.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = slice.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            (min, max)
        })
        .collect()
}

impl Render for Waveform {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        let peaks = self.peaks.clone();
        let color = self.color;
        let w = self.width;
        let h = self.height;
        let mid = h / 2.0;

        canvas(
            move |bounds, cx| {
                if peaks.is_empty() {
                    // Draw center line
                    let mut path = Path::new(point(bounds.origin.x, bounds.origin.y + px(mid)));
                    path.line_to(point(bounds.origin.x + px(w), bounds.origin.y + px(mid)));
                    cx.paint_path(path, rgba(0x1a2a3aff));
                    return;
                }

                let col_w = w / peaks.len() as f32;

                // Draw waveform as filled columns
                for (i, (min, max)) in peaks.iter().enumerate() {
                    let x = bounds.origin.x + px(i as f32 * col_w);
                    let y_top = bounds.origin.y + px(mid - max.clamp(-1.0, 1.0) * mid);
                    let y_bot = bounds.origin.y + px(mid - min.clamp(-1.0, 1.0) * mid);
                    let col_h = (y_bot - y_top).max(px(1.0));

                    let mut path = Path::new(point(x, y_top));
                    path.line_to(point(x + px(col_w.max(1.0)), y_top));
                    path.line_to(point(x + px(col_w.max(1.0)), y_bot));
                    path.line_to(point(x, y_bot));
                    path.line_to(point(x, y_top));
                    let _ = col_h;
                    cx.paint_path(path, color);
                }
            },
            |_, _| {},
        )
        .w(px(w))
        .h(px(h))
    }
}
