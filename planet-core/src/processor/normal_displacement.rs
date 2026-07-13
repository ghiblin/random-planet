use rand::RngExt;
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::normal_noise_range::NormalNoiseRange;

pub(crate) fn normal_displacement(range: NormalNoiseRange) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, a, b, point| {
        let edge_length = b.position.sub(a.position).length();
        let delta = edge_length * rng.random_range(range.low()..=range.high());
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

        // Edge length |b - a| = sqrt(2), so the drawn 0.05 fraction becomes sqrt(2) * 0.05.
        let expected = Vec3::new(0.5, 0.5, 2.0_f32.sqrt() * 0.05);
        assert!(
            (result.position.z - expected.z).abs() < 1e-5,
            "expected z {}, got {}",
            expected.z,
            result.position.z
        );
        assert_eq!(result.position.x, expected.x);
        assert_eq!(result.position.y, expected.y);
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

    #[test]
    fn zero_length_edge_leaves_position_unchanged_even_for_a_wide_range() {
        let mut rng = Pcg32::seed_from_u64(7);
        let coincident = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(1.5, 0.0, 0.0),
        };
        let range = NormalNoiseRange::new(-10.0, 10.0).expect("valid range");

        let result = normal_displacement(range)(&mut rng, &coincident, &coincident, point);

        assert_eq!(result.position, point.position);
    }
}
