use nalgebra::Point2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line(pub Point2<f32>, pub Point2<f32>);