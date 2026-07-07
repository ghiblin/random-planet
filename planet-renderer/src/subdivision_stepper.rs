use planet_core::mesh::{Mesh, MeshError};
use planet_core::subdivide::{SubdivisionStrategy, subdivide};

#[derive(Debug)]
pub struct SubdivisionStepper {
    mesh: Mesh,
    rounds_completed: u32,
    max_depth: u32,
}

impl SubdivisionStepper {
    pub fn new(base_mesh: Mesh, max_depth: u32) -> SubdivisionStepper {
        SubdivisionStepper {
            mesh: base_mesh,
            rounds_completed: 0,
            max_depth,
        }
    }

    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn rounds_completed(&self) -> u32 {
        self.rounds_completed
    }

    pub fn can_step(&self) -> bool {
        self.rounds_completed < self.max_depth
    }

    pub fn step(&mut self, strategy: &mut dyn SubdivisionStrategy) -> Result<bool, MeshError> {
        if !self.can_step() {
            return Ok(false);
        }
        self.mesh = subdivide(&self.mesh, 1, strategy)?;
        self.rounds_completed += 1;
        Ok(true)
    }
}
