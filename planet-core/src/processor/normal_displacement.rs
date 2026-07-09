use rand::RngExt;
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::normal_noise_range::NormalNoiseRange;

pub(crate) fn normal_displacement(range: NormalNoiseRange) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, a, b, point| {
        let delta = rng.random_range(range.low()..=range.high());
        match a.position.cross(b.position).normalized() {
            Some(normal) => Vertex {
                position: point.position.add(normal.scale(delta)),
            },
            None => point,
        }
    })
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_pcg::Pcg32;

    use super::normal_displacement;
    use crate::geometry::mesh::Vertex;
    use crate::geometry::vec3::Vec3;
    use crate::subdivision::normal_noise_range::NormalNoiseRange;

    #[test]
    fn degenerate_cross_product_leaves_point_unchanged() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(2.0, 0.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(1.5, 0.0, 0.0),
        };
        let range = NormalNoiseRange::new(0.05, 0.05).expect("valid range");

        let result = normal_displacement(range)(&mut rng, &a, &b, point);

        assert_eq!(result.position, point.position);
    }

    #[test]
    fn displaces_along_the_edge_plane_normal_by_the_drawn_delta() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(0.5, 0.5, 0.0),
        };
        let range = NormalNoiseRange::new(0.05, 0.05).expect("valid range");

        let result = normal_displacement(range)(&mut rng, &a, &b, point);

        let expected = Vec3::new(0.5, 0.5, 0.05);
        assert_eq!(result.position, expected);
    }

    #[test]
    fn zero_width_range_leaves_position_bit_identical() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(0.5, 0.5, 0.0),
        };
        let range = NormalNoiseRange::new(0.0, 0.0).expect("valid range");

        let result = normal_displacement(range)(&mut rng, &a, &b, point);

        assert_eq!(result.position, point.position);
    }
}
