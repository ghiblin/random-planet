use crate::geometry::mesh::{Mesh, MeshError};

pub(crate) type MeshProcessor = Box<dyn Fn(&Mesh) -> Result<Mesh, MeshError>>;
