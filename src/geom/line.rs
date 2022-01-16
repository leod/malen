use nalgebra::{Point2, Vector2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub Point2<f32>, pub Point2<f32>);

impl Line {
    pub fn delta(&self) -> Vector2<f32> {
        self.1 - self.0
    }
}
