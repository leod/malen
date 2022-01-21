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

        let dx = off_x * Vector2::new(cos, sin);
        let dy = off_y * Vector2::new(-sin, cos);

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
}
