use nalgebra::{Point2, Vector2};

use super::Line;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotatedRect {
    pub center: Point2<f32>,
    pub size: Vector2<f32>,
    pub angle: f32,
}

impl RotatedRect {
    pub fn translate(self, delta: Vector2<f32>) -> Self {
        Self {
            center: self.center + delta,
            size: self.size,
            angle: self.angle,
        }
    }

    pub fn rotate(self, angle: f32) -> RotatedRect {
        RotatedRect {
            center: self.center,
            size: self.size,
            angle: self.angle + angle,
        }
    }

    fn dxdy(self) -> (Vector2<f32>, Vector2<f32>) {
        let off_x = self.size.x / 2.0;
        let off_y = self.size.y / 2.0;

        let cos = self.angle.cos();
        let sin = self.angle.sin();

        let dx = Vector2::new(cos * off_x, sin * off_x);
        let dy = Vector2::new(-sin * off_y, cos * off_y);

        (dx, dy)
    }

    pub fn corners(self) -> [Point2<f32>; 4] {
        let (dx, dy) = self.dxdy();

        [
            self.center - dx - dy,
            self.center + dx - dy,
            self.center + dx + dy,
            self.center - dx + dy,
        ]
    }

    pub fn edges(&self) -> [Line; 4] {
        let corners = self.corners();

        [
            Line(corners[0], corners[1]),
            Line(corners[1], corners[2]),
            Line(corners[2], corners[3]),
            Line(corners[3], corners[0]),
        ]
    }

    pub fn caps(&self) -> [Line; 4] {
        let corners = self.corners();
        let (dx, dy) = self.dxdy();

        let dx = (0.25 * dx.norm()).min(30.0) * dx.normalize();
        let dy = (0.25 * dy.norm()).min(30.0) * dy.normalize();

        [
            Line(corners[0] + dx, corners[0] + dy),
            Line(corners[1] - dx, corners[1] + dy),
            Line(corners[2] - dx, corners[2] - dy),
            Line(corners[3] + dx, corners[3] - dy),
        ]
    }
}
