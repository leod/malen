use nalgebra::Point2;

use super::{shape::Shape, Circle, Rect};

pub fn rect_circle_overlap(rect: Rect, circle: Circle) -> bool {
    // https://math.stackexchange.com/questions/227494/do-an-axis-aligned-rectangle-and-a-circle-overlap

    let p_star = Point2::new(
        circle
            .center
            .x
            .max(rect.top_left().x)
            .min(rect.bottom_right().x),
        circle
            .center
            .y
            .max(rect.top_left().y)
            .min(rect.bottom_right().y),
    );

    (p_star - circle.center).norm_squared() < circle.radius * circle.radius
}

pub fn rect_rect_overlap(r1: Rect, r2: Rect) -> bool {
    // https://gamedev.stackexchange.com/questions/586/what-is-the-fastest-way-to-work-out-2d-bounding-box-intersection

    (r1.center.x - r2.center.x).abs() * 2.0 < r1.size.x + r2.size.x
        && (r1.center.y - r2.center.y).abs() * 2.0 < r1.size.y + r2.size.y
}

pub fn circle_circle_overlap(c1: Circle, c2: Circle) -> bool {
    (c1.center - c2.center).norm_squared() <= (c1.radius + c2.radius).powi(2)
}

pub fn shape_shape_overlap(s1: &Shape, s2: &Shape) -> bool {
    match (s1, s2) {
        (Shape::Rect(r1), Shape::Rect(r2)) => rect_rect_overlap(*r1, *r2),
        (Shape::Circle(r1), Shape::Circle(r2)) => circle_circle_overlap(*r1, *r2),
        (Shape::Rect(r), Shape::Circle(c)) => rect_circle_overlap(*r, *c),
        (Shape::Circle(c), Shape::Rect(r)) => rect_circle_overlap(*r, *c),
        (_, Shape::RotatedRect(_)) | (Shape::RotatedRect(_), _) => unimplemented!(),
    }
}
