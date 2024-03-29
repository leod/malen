use nalgebra::{Point2, Vector2};
use rand::Rng;

use super::{Line, RotatedRect};

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

    pub fn left_x(self) -> f32 {
        self.center.x - self.size.x / 2.0
    }

    pub fn right_x(self) -> f32 {
        self.center.x + self.size.x / 2.0
    }

    pub fn top_y(self) -> f32 {
        self.center.y - self.size.y / 2.0
    }

    pub fn bottom_y(self) -> f32 {
        self.center.y + self.size.y / 2.0
    }

    pub fn edges(self) -> [Line; 4] {
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

    pub fn enlarge(mut self, add: Vector2<f32>) -> Self {
        self.size += add;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.size *= scale;
        self
    }

    pub fn contains_point(self, p: Point2<f32>) -> bool {
        (self.left_x()..=self.right_x()).contains(&p.x)
            && (self.top_y()..=self.bottom_y()).contains(&p.y)
    }

    pub fn sample<R: Rng>(self, rng: &mut R) -> Point2<f32> {
        let x = rng.gen_range(-0.5, 0.5) * self.size.x;
        let y = rng.gen_range(-0.5, 0.5) * self.size.y;

        self.center + Vector2::new(x, y)
    }
}
