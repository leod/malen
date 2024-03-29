use nalgebra::{Point2, Vector2};

use super::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point2<f32>,
    pub radius: f32,
}

impl Circle {
    pub fn points(self, angle: f32, n: usize) -> impl Iterator<Item = Point2<f32>> {
        (0..n).map(move |i| {
            let phi = angle + i as f32 / n as f32 * 2.0 * std::f32::consts::PI;

            self.center + self.radius * Vector2::new(phi.cos(), phi.sin())
        })
    }

    pub fn bounding_rect(self) -> Rect {
        Rect {
            center: self.center,
            size: Vector2::new(2.0 * self.radius, 2.0 * self.radius),
        }
    }
}
