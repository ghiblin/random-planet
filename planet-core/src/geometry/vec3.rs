#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Vec3 {
        Vec3 { x, y, z }
    }

    pub fn add(&self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    pub fn sub(&self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    pub fn scale(&self, factor: f32) -> Vec3 {
        Vec3::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length(&self) -> f32 {
        self.dot(*self).sqrt()
    }

    pub fn normalized(&self) -> Option<Vec3> {
        let length = self.length();
        if length == 0.0 {
            None
        } else {
            Some(self.scale(1.0 / length))
        }
    }
}
