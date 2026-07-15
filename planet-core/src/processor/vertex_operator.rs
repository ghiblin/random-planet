use rand_pcg::Pcg32;

use crate::geometry::vertex::Vertex;

pub(crate) type VertexOperator = Box<dyn Fn(&mut Pcg32, &Vertex, &Vertex, Vertex) -> Vertex>;
