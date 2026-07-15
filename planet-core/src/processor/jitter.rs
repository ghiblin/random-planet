use rand::RngExt;

use crate::geometry::vertex::Vertex;

use super::vertex_operator::VertexOperator;

const SPLIT_POINT_VARIANCE: f32 = 0.05;
const NORMAL_OFFSET_FRACTION: f32 = 0.03;

pub(crate) fn jitter() -> VertexOperator {
    Box::new(|rng, a, b, _exact_midpoint| {
        let edge = b.position.sub(a.position);
        let edge_length = edge.length();
        let t = rng.random_range((0.5 - SPLIT_POINT_VARIANCE)..=(0.5 + SPLIT_POINT_VARIANCE));
        let mut position = a.position.add(edge.scale(t));
        if let Some(normal) = a.position.cross(b.position).normalized() {
            let normal_delta =
                rng.random_range(-NORMAL_OFFSET_FRACTION..=NORMAL_OFFSET_FRACTION) * edge_length;
            position = position.add(normal.scale(normal_delta));
        }
        Vertex::at(position)
    })
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_pcg::Pcg32;

    use super::jitter;
    use crate::geometry::vec3::Vec3;
    use crate::geometry::vertex::Vertex;

    fn vertex(x: f32, y: f32, z: f32) -> Vertex {
        Vertex::at(Vec3::new(x, y, z))
    }

    fn exact_midpoint(a: &Vertex, b: &Vertex) -> Vertex {
        Vertex::at(a.position.add(b.position).scale(0.5))
    }

    #[test]
    fn displaces_the_split_point_away_from_the_exact_midpoint() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = vertex(1.0, 0.0, 0.0);
        let b = vertex(0.0, 1.0, 0.0);
        let midpoint = exact_midpoint(&a, &b);

        let result = jitter()(&mut rng, &a, &b, midpoint.clone());

        assert_ne!(result.position, midpoint.position);
    }

    #[test]
    fn displacement_from_the_exact_midpoint_is_bounded_by_edge_length() {
        let a = vertex(1.0, 0.0, 0.0);
        let b = vertex(0.0, 1.0, 0.0);
        let midpoint = exact_midpoint(&a, &b);
        let edge_length = b.position.sub(a.position).length();
        let bound = 0.06 * edge_length;

        for seed in 0..50 {
            let mut rng = Pcg32::seed_from_u64(seed);
            let result = jitter()(&mut rng, &a, &b, midpoint.clone());
            let distance = result.position.sub(midpoint.position).length();
            assert!(
                distance <= bound,
                "seed {seed}: displacement {distance} exceeds bound {bound}"
            );
        }
    }

    #[test]
    fn skips_the_normal_offset_when_the_edge_touches_the_origin() {
        let mut rng = Pcg32::seed_from_u64(7);
        let origin = vertex(0.0, 0.0, 0.0);
        let b = vertex(2.0, 0.0, 0.0);
        let midpoint = exact_midpoint(&origin, &b);

        let result = jitter()(&mut rng, &origin, &b, midpoint);

        // cross(origin, b) is the zero vector, so no normal offset is applied;
        // the result must still lie exactly on the edge (y and z stay 0).
        assert_eq!(result.position.y, 0.0);
        assert_eq!(result.position.z, 0.0);
        assert!(result.position.x.is_finite());
    }
}
