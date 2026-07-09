use rand_pcg::Pcg32;

use crate::geometry::mesh::Vertex;
use crate::processor::vertex_operator::VertexOperator;

// Applies `first`, then `second` — left-to-right, not the right-to-left order
// mathematical f∘g notation implies.
pub(crate) fn compose(first: VertexOperator, second: VertexOperator) -> VertexOperator {
    Box::new(
        move |rng: &mut Pcg32, a: &Vertex, b: &Vertex, point: Vertex| {
            let point = first(rng, a, b, point);
            second(rng, a, b, point)
        },
    )
}

#[cfg(test)]
mod tests {
    use rand::{RngExt, SeedableRng};
    use rand_pcg::Pcg32;

    use super::compose;
    use crate::geometry::mesh::Vertex;
    use crate::geometry::vec3::Vec3;
    use crate::processor::vertex_operator::VertexOperator;

    #[test]
    fn applies_first_then_second() {
        let mut rng = Pcg32::seed_from_u64(7);
        let a = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(1.0, 0.0, 0.0),
        };
        let double: VertexOperator = Box::new(|_rng, _a, _b, point: Vertex| Vertex {
            position: point.position.scale(2.0),
        });
        let add_one_x: VertexOperator = Box::new(|_rng, _a, _b, point: Vertex| Vertex {
            position: point.position.add(Vec3::new(1.0, 0.0, 0.0)),
        });

        let result = compose(double, add_one_x)(&mut rng, &a, &b, point);

        // first `double` (1 -> 2), then `add_one_x` (2 -> 3) — not 4, which is what
        // applying `add_one_x` before `double` would produce.
        assert_eq!(result.position, Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn threads_the_same_rng_through_both_operators_in_sequence() {
        let mut expected_rng = Pcg32::seed_from_u64(42);
        let first_draw: f32 = expected_rng.random_range(0.0..=1.0);
        let second_draw: f32 = expected_rng.random_range(0.0..=1.0);

        let mut rng = Pcg32::seed_from_u64(42);
        let a = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let b = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let point = Vertex {
            position: Vec3::new(0.0, 0.0, 0.0),
        };
        let draw_and_add_x: VertexOperator = Box::new(|rng: &mut Pcg32, _a, _b, point: Vertex| {
            let delta = rng.random_range(0.0..=1.0);
            Vertex {
                position: point.position.add(Vec3::new(delta, 0.0, 0.0)),
            }
        });
        let draw_and_add_x_again: VertexOperator =
            Box::new(|rng: &mut Pcg32, _a, _b, point: Vertex| {
                let delta = rng.random_range(0.0..=1.0);
                Vertex {
                    position: point.position.add(Vec3::new(delta, 0.0, 0.0)),
                }
            });

        let result = compose(draw_and_add_x, draw_and_add_x_again)(&mut rng, &a, &b, point);

        assert_eq!(
            result.position,
            Vec3::new(first_draw + second_draw, 0.0, 0.0)
        );
    }
}
