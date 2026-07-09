use crate::processor::vertex_operator::VertexOperator;

pub(crate) fn identity() -> VertexOperator {
    Box::new(|_rng, _a, _b, point| point)
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_pcg::Pcg32;

    use super::identity;
    use crate::geometry::mesh::Vertex;
    use crate::geometry::vec3::Vec3;

    #[test]
    fn identity_returns_the_passed_vertex_unchanged() {
        let mut rng = Pcg32::seed_from_u64(0);
        let a = Vertex {
            position: Vec3::new(1.0, 2.0, 3.0),
        };
        let b = Vertex {
            position: Vec3::new(4.0, 5.0, 6.0),
        };
        let point = Vertex {
            position: Vec3::new(7.0, 8.0, 9.0),
        };

        let result = identity()(&mut rng, &a, &b, point);

        assert_eq!(result.position, point.position);
    }
}
