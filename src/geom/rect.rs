use nalgebra::{Point2, Vector2};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
}

impl Rect {
    pub fn zero_to_one() -> Self {
        Self::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0))
    }

    pub fn from_top_left(top_left: Point2<f32>, size: Vector2<f32>) -> Self {
        Self {
            center: top_left + size / 2.0,
            size,
        }
    }

    pub fn from_bottom_left(bottom_left: Point2<f32>, size: Vector2<f32>) -> Self {
        Self {
            center: bottom_left + Vector2::new(size.x, -size.y) / 2.0,
            size,
        }
    }

    pub fn translate(self, delta: Vector2<f32>) -> Self {
        Self {
            center: self.center + delta,
            size: self.size,
        }
    }
}
