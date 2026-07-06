#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

impl Camera {
    pub const MIN_DISTANCE: f32 = 2.0;
    pub const MAX_DISTANCE: f32 = 20.0;
    pub const DEFAULT_DISTANCE: f32 = 5.0;
    pub const MAX_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.01;

    pub fn at_distance(distance: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance,
        }
    }

    pub fn orbit(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch = (self.pitch + delta_pitch).clamp(-Self::MAX_PITCH, Self::MAX_PITCH);
    }

    pub fn zoom(&mut self, scroll_delta: f32) {
        self.distance =
            (self.distance + scroll_delta).clamp(Self::MIN_DISTANCE, Self::MAX_DISTANCE);
    }

    pub fn view_projection_matrix(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let _ = aspect_ratio;
        unimplemented!()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::at_distance(Self::DEFAULT_DISTANCE)
    }
}
