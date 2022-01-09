use nalgebra::{Point2, Vector2};

use super::Line;

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn rotate(self, angle: f32) -> RotatedRect {
        RotatedRect {
            center: self.center,
            size: self.size,
            angle,
        }
    }

    pub fn to_rotated_rect(self) -> RotatedRect {
        self.rotate(0.0)
    }

    pub fn corners(self) -> [Point2<f32>; 4] {
        let dx = Vector2::new(self.size.x / 2.0, 0.0);
        let dy = Vector2::new(0.0, self.size.y / 2.0);

        [
            self.center - dx - dy,
            self.center + dx - dy,
            self.center + dx + dy,
            self.center - dx + dy,
        ]
    }

    pub fn top_left(self) -> Point2<f32> {
        self.corners()[0]
    }

    pub fn top_right(self) -> Point2<f32> {
        self.corners()[1]
    }

    pub fn bottom_right(self) -> Point2<f32> {
        self.corners()[2]
    }

    pub fn bottom_left(self) -> Point2<f32> {
        self.corners()[3]
    }

    pub fn lines(self) -> [Line; 4] {
        let corners = self.corners();

        [
            Line(corners[0], corners[1]),
            Line(corners[1], corners[2]),
            Line(corners[2], corners[3]),
            Line(corners[3], corners[0]),
        ]
    }

    pub fn diagonals(self) -> [Line; 2] {
        let corners = self.corners();

        [Line(corners[0], corners[2]), Line(corners[1], corners[3])]
    }

    pub fn caps(self) -> [Line; 4] {
        let corners = self.corners();

        let dx = Vector2::new((0.25 * self.size.x).min(30.0), 0.0);
        let dy = Vector2::new(0.0, (0.25 * self.size.y).min(30.0));

        [
            Line(corners[0] + dx, corners[0] + dy),
            Line(corners[1] - dx, corners[1] + dy),
            Line(corners[2] - dx, corners[2] - dy),
            Line(corners[3] + dx, corners[3] - dy),
        ]
    }

    pub fn enlarge(mut self, add: Vector2<f32>) -> Rect {
        self.size += add;
        self
    }
}

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

    pub fn lines(&self) -> [Line; 4] {
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
