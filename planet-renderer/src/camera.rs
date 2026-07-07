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
        let eye = [
            self.distance * self.pitch.cos() * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * self.pitch.cos() * self.yaw.cos(),
        ];
        let view = look_at_rh(eye, [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let projection = perspective_rh_zo(std::f32::consts::FRAC_PI_4, aspect_ratio, 0.1, 100.0);
        mat4_mul(projection, view)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::at_distance(Self::DEFAULT_DISTANCE)
    }
}

type Vec3 = [f32; 3];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(v: Vec3) -> Vec3 {
    let len = dot(v, v).sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Right-handed look-at view matrix, stored column-major (outer index = column).
fn look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> [[f32; 4]; 4] {
    let forward = normalize(sub(target, eye));
    let right = normalize(cross(forward, up));
    let true_up = cross(right, forward);

    [
        [right[0], true_up[0], -forward[0], 0.0],
        [right[1], true_up[1], -forward[1], 0.0],
        [right[2], true_up[2], -forward[2], 0.0],
        [-dot(right, eye), -dot(true_up, eye), dot(forward, eye), 1.0],
    ]
}

/// Right-handed perspective projection with a `0..1` depth range (wgpu clip space),
/// stored column-major (outer index = column).
fn perspective_rh_zo(fov_y_radians: f32, aspect_ratio: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y_radians / 2.0).tan();
    [
        [f / aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, (near * far) / (near - far), 0.0],
    ]
}

/// Column-major 4x4 matrix multiplication: `a * b`.
fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0f32; 4]; 4];
    for (col, result_col) in result.iter_mut().enumerate() {
        for (row, cell) in result_col.iter_mut().enumerate() {
            *cell = (0..4).map(|k| a[k][row] * b[col][k]).sum();
        }
    }
    result
}
