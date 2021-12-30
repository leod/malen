use nalgebra::{Point2, Vector2};

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
            angle,
        }
    }

    pub fn corners(self) -> [Point2<f32>; 4] {
        let off_x = self.size.x / 2.0;
        let off_y = self.size.y / 2.0;

        let cos = self.angle.cos();
        let sin = self.angle.sin();

        let dx = Vector2::new(cos * off_x, sin * off_x);
        let dy = Vector2::new(-sin * off_y, cos * off_y);

        [
            self.center - dx - dy,
            self.center + dx - dy,
            self.center + dx + dy,
            self.center - dx + dy,
        ]
    }
}
