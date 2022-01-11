use super::{Circle, Rect, RotatedRect};

#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    RotatedRect(RotatedRect),
    Circle(Circle),
}
