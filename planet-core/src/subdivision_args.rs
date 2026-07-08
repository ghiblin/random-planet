use crate::mesh::Mesh;
use crate::steps::Steps;
use crate::subdivision_mode::SubdivisionMode;

pub type UpdateCallback = Box<dyn FnMut(&Mesh, usize)>;

pub struct SubdivisionArgs {
    pub(crate) steps: Steps,
    pub(crate) mode: SubdivisionMode,
    pub(crate) update_cb: Option<UpdateCallback>,
}

impl SubdivisionArgs {
    pub fn new(
        steps: Option<Steps>,
        mode: Option<SubdivisionMode>,
        update_cb: Option<UpdateCallback>,
    ) -> SubdivisionArgs {
        SubdivisionArgs {
            steps: steps.unwrap_or_default(),
            mode: mode.unwrap_or_default(),
            update_cb,
        }
    }

    pub fn steps(&self) -> Steps {
        self.steps
    }

    pub fn mode(&self) -> SubdivisionMode {
        self.mode
    }
}
