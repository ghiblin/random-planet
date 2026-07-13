use rand::RngExt;
use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;
use crate::subdivision::elevation_noise_range::ElevationNoiseRange;

pub(crate) const MIN_VERTEX_RADIUS: f32 = 0.05;

pub(crate) fn radial_displacement(range: ElevationNoiseRange) -> VertexOperator {
    Box::new(move |rng: &mut Pcg32, a, b, point| {
        let radius = point.position.length();
        if radius == 0.0 {
            return point;
        }
        let edge_length = b.position.sub(a.position).length();
        let delta = edge_length * rng.random_range(range.low()..=range.high());
        let new_radius = (radius + delta).max(MIN_VERTEX_RADIUS);
        Vertex {
            position: point.position.scale(new_radius / radius),
        }
    })
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_pcg::Pcg32;

    use super::{MIN_VERTEX_RADIUS, radial_displacement};
    use crate::geometry::mesh::Vertex;
    use crate::geometry::vec3::Vec3;
    use crate::subdivision::elevation_noise_range::ElevationNoiseRange;

    #[test]
    fn zero_radius_point_is_returned_unchanged() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(-1.0, 0.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let range = ElevationNoiseRange::default();

        let result = radial_displacement(range)(&mut rng, &a, &b, point);

        assert_eq!(result.position, point.position);
    }

    #[test]
    fn radius_is_clamped_to_min_vertex_radius() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let range = ElevationNoiseRange::new(-10.0, -10.0).expect("valid range");

        let result = radial_displacement(range)(&mut rng, &a, &b, point);

        assert!((result.position.length() - MIN_VERTEX_RADIUS).abs() < 1e-6);
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
            position: Vec3::new(2.0, 0.0, 0.0),
        };
        let range = ElevationNoiseRange::new(0.0, 0.0).expect("valid range");

        let result = radial_displacement(range)(&mut rng, &a, &b, point);

        assert_eq!(result.position, point.position);
    }

    #[test]
    fn displacement_scales_with_the_edge_length() {
        let mut rng = Pcg32::seed_from_u64(7);
        // Edge length |b - a| = sqrt(2).
        let a = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 1.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let range = ElevationNoiseRange::new(0.1, 0.1).expect("valid range");

        let result = radial_displacement(range)(&mut rng, &a, &b, point);

        let expected_radius = 1.0 + 2.0_f32.sqrt() * 0.1;
        assert!(
            (result.position.length() - expected_radius).abs() < 1e-5,
            "expected radius {expected_radius}, got {}",
            result.position.length()
        );
    }

    #[test]
    fn zero_length_edge_leaves_position_unchanged_even_for_a_wide_range() {
        let mut rng = Pcg32::seed_from_u64(7);
        let coincident = Vertex {
            position: Vec3::new(3.0, 4.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(3.0, 4.0, 0.0),
        };
        let range = ElevationNoiseRange::new(-10.0, 10.0).expect("valid range");

        let result = radial_displacement(range)(&mut rng, &coincident, &coincident, point);

        assert_eq!(result.position, point.position);
    }
}
