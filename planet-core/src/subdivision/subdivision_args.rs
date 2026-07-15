use super::seed::Seed;
use super::steps::Steps;
use super::subdivision_mode::SubdivisionMode;
use crate::geometry::mesh::Mesh;

pub type UpdateCallback = Box<dyn FnMut(&Mesh, usize)>;

pub struct SubdivisionArgs {
    pub(crate) steps: Steps,
    pub(crate) mode: SubdivisionMode,
    pub(crate) seed: Seed,
    pub(crate) update_cb: Option<UpdateCallback>,
}

impl SubdivisionArgs {
    pub fn new(
        steps: Option<Steps>,
        mode: Option<SubdivisionMode>,
        seed: Option<Seed>,
        update_cb: Option<UpdateCallback>,
    ) -> SubdivisionArgs {
        SubdivisionArgs {
            steps: steps.unwrap_or_default(),
            mode: mode.unwrap_or_default(),
            seed: seed.unwrap_or_default(),
            update_cb,
        }
    }

    pub fn steps(&self) -> Steps {
        self.steps
    }

    pub fn mode(&self) -> SubdivisionMode {
        self.mode
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }
}
