use std::collections::VecDeque;

use crate::gpu::buffers::PackedFrame;

pub const FRAME_INTERVAL_MS: f64 = 150.0;

#[derive(Debug, Clone, Default)]
pub struct GrowthAnimation {
    revealed: Vec<PackedFrame>,
    pending: VecDeque<PackedFrame>,
    last_advance_ms: Option<f64>,
}

impl GrowthAnimation {
    pub fn new() -> GrowthAnimation {
        GrowthAnimation::default()
    }

    pub fn push_frame(&mut self, frame: PackedFrame, now_ms: f64) {
        if self.revealed.is_empty() {
            self.revealed.push(frame);
            self.last_advance_ms = Some(now_ms);
        } else {
            self.pending.push_back(frame);
        }
    }

    pub fn tick(&mut self, now_ms: f64) -> bool {
        let Some(last_advance_ms) = self.last_advance_ms else {
            return false;
        };
        if now_ms - last_advance_ms < FRAME_INTERVAL_MS {
            return false;
        }
        let Some(frame) = self.pending.pop_front() else {
            return false;
        };
        self.revealed.push(frame);
        self.last_advance_ms = Some(now_ms);
        true
    }

    pub fn current(&self) -> Option<&PackedFrame> {
        self.revealed.last()
    }
}
