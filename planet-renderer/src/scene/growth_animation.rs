use planet_core::color::rgb::Rgb;
use planet_core::geometry::mesh::Mesh;

pub const FRAME_INTERVAL_MS: f64 = 150.0;

#[derive(Debug, Clone)]
pub struct GrowthAnimation {
    frames: Vec<(Mesh, Vec<Rgb>)>,
    current_frame: usize,
    last_advance_ms: f64,
}

impl GrowthAnimation {
    pub fn new(frames: Vec<(Mesh, Vec<Rgb>)>, started_ms: f64) -> GrowthAnimation {
        GrowthAnimation {
            frames,
            current_frame: 0,
            last_advance_ms: started_ms,
        }
    }

    pub fn current(&self) -> &(Mesh, Vec<Rgb>) {
        &self.frames[self.current_frame]
    }

    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    pub fn tick(&mut self, now_ms: f64) -> bool {
        let has_next_frame = self.current_frame + 1 < self.frames.len();
        let interval_elapsed = now_ms - self.last_advance_ms >= FRAME_INTERVAL_MS;
        if has_next_frame && interval_elapsed {
            self.current_frame += 1;
            self.last_advance_ms = now_ms;
            true
        } else {
            false
        }
    }
}
