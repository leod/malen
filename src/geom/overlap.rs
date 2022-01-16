use nalgebra::{Point2, Rotation2, Vector2};

use super::{shape::Shape, Circle, Rect, RotatedRect};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Overlap(Vector2<f32>);

impl Overlap {
    pub fn resolution(self) -> Vector2<f32> {
        self.0
    }

    pub fn neg(self) -> Overlap {
        Overlap(-self.0)
    }
}

pub fn rect_circle_overlap(r: Rect, c: Circle) -> Option<Overlap> {
    // https://math.stackexchange.com/questions/227494/do-an-axis-aligned-rectangle-and-a-circle-overlap

    let p_star = Point2::new(
        c.center.x.max(r.top_left().x).min(r.bottom_right().x),
        c.center.y.max(r.top_left().y).min(r.bottom_right().y),
    );

    let delta = p_star - c.center;
    let dist_sq = delta.norm_squared();

    if dist_sq < c.radius * c.radius {
        let dist = dist_sq.sqrt();
        let normal = if dist < 0.000001 {
            Vector2::new(-1.0, 0.0)
        } else {
            delta / dist
        };

        Some(Overlap(-normal * (c.radius - dist + 1.0)))
    } else {
        None
    }
}

pub fn rotated_rect_circle_overlap(r: RotatedRect, c: Circle) -> Option<Overlap> {
    let rotation = Rotation2::new(r.angle);
    let inv_rotation = Rotation2::new(-r.angle);

    let rect_origin = Rect {
        center: Point2::origin(),
        size: r.size,
    };
    let circle_shifted = Circle {
        center: inv_rotation * (c.center - r.center.coords),
        radius: c.radius,
    };

    rect_circle_overlap(rect_origin, circle_shifted).map(|overlap| Overlap(rotation * overlap.0))
}

pub fn rect_rect_overlap(r1: Rect, r2: Rect) -> Option<Overlap> {
    // https://gamedev.stackexchange.com/questions/586/what-is-the-fastest-way-to-work-out-2d-bounding-box-intersection

    if (r1.center.x - r2.center.x).abs() * 2.0 < r1.size.x + r2.size.x
        && (r1.center.y - r2.center.y).abs() * 2.0 < r1.size.y + r2.size.y
    {
        Some(Overlap(Vector2::zeros()))
    } else {
        None
    }
}

pub fn circle_circle_overlap(c1: Circle, c2: Circle) -> Option<Overlap> {
    if (c1.center - c2.center).norm_squared() <= (c1.radius + c2.radius).powi(2) {
        Some(Overlap(Vector2::zeros()))
    } else {
        None
    }
}

pub fn shape_shape_overlap(s1: &Shape, s2: &Shape) -> Option<Overlap> {
    match (s1, s2) {
        (Shape::Rect(r1), Shape::Rect(r2)) => rect_rect_overlap(*r1, *r2),
        (Shape::Circle(r1), Shape::Circle(r2)) => circle_circle_overlap(*r1, *r2).map(Overlap::neg),

        (Shape::Rect(r), Shape::Circle(c)) => rect_circle_overlap(*r, *c),
        (Shape::Circle(c), Shape::Rect(r)) => rect_circle_overlap(*r, *c).map(Overlap::neg),

        (Shape::RotatedRect(r), Shape::Circle(c)) => rotated_rect_circle_overlap(*r, *c),
        (Shape::Circle(c), Shape::RotatedRect(r)) => {
            rotated_rect_circle_overlap(*r, *c).map(Overlap::neg)
        }

        (_, Shape::RotatedRect(_)) | (Shape::RotatedRect(_), _) => None, //unimplemented!(),
    }
}
