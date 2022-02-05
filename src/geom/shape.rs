use super::{Circle, Rect, RotatedRect};

#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    RotatedRect(RotatedRect),
    Circle(Circle),
}

impl Shape {
    pub fn bounding_rect(&self) -> Rect {
        use Shape::*;

        match self {
            Rect(r) => *r,
            RotatedRect(r) => r.bounding_rect(),
            Circle(c) => c.bounding_rect(),
        }
    }
}
