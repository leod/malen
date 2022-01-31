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

        Some(Overlap(normal * (c.radius - dist + 1.0)))
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
        // TODO: rect_rect_overlap
        Some(Overlap(Vector2::zeros()))
    } else {
        None
    }
}

pub fn rotated_rect_rotated_rect_overlap(r1: RotatedRect, r2: RotatedRect) -> Option<Overlap> {
    let mut min_dist_axis = None;

    for line in r1.edges().into_iter().chain(r2.edges()) {
        let axis = Vector2::new(-line.delta().y, line.delta().x).normalize();

        let r1_proj = AxisProj::project_rotated_rect(axis, r1);
        let r2_proj = AxisProj::project_rotated_rect(axis, r2);

        let dist = r1_proj.interval_distance(r2_proj);

        if dist > 0.0 {
            // By the separating axis theorem, the polygons do not overlap.
            return None;
        }

        // Keep the axis with the minimum interval distance.
        if min_dist_axis.map_or(true, |(min_dist, _)| dist.abs() < min_dist) {
            min_dist_axis = Some((
                dist.abs(),
                (r1.center - r2.center).dot(&axis).signum() * axis,
            ));
        }
    }

    min_dist_axis.map(|(min_dist, min_axis)| Overlap(min_dist * min_axis))
}

pub fn circle_circle_overlap(c1: Circle, c2: Circle) -> Option<Overlap> {
    let delta = c1.center - c2.center;
    let dist_sq = delta.norm_squared();

    if dist_sq <= (c1.radius + c2.radius).powi(2) {
        let dist = dist_sq.sqrt();
        let normal = if dist < 0.000001 {
            Vector2::new(-1.0, 0.0)
        } else {
            delta / dist
        };

        Some(Overlap((c1.radius + c2.radius - dist) * normal))
    } else {
        None
    }
}

pub fn shape_shape_overlap(s1: &Shape, s2: &Shape) -> Option<Overlap> {
    match (s1, s2) {
        (Shape::Rect(r1), Shape::Rect(r2)) => rect_rect_overlap(*r1, *r2),
        (Shape::RotatedRect(r1), Shape::RotatedRect(r2)) => {
            rotated_rect_rotated_rect_overlap(*r1, *r2)
        }
        (Shape::Circle(r1), Shape::Circle(r2)) => circle_circle_overlap(*r1, *r2),

        (Shape::Rect(r1), Shape::RotatedRect(r2)) => {
            rotated_rect_rotated_rect_overlap(r1.to_rotated_rect(), *r2)
        }
        (Shape::RotatedRect(r1), Shape::Rect(r2)) => {
            rotated_rect_rotated_rect_overlap(*r1, r2.to_rotated_rect())
        }

        (Shape::Rect(r), Shape::Circle(c)) => rect_circle_overlap(*r, *c),
        (Shape::Circle(c), Shape::Rect(r)) => rect_circle_overlap(*r, *c).map(Overlap::neg),

        (Shape::RotatedRect(r), Shape::Circle(c)) => rotated_rect_circle_overlap(*r, *c),
        (Shape::Circle(c), Shape::RotatedRect(r)) => {
            rotated_rect_circle_overlap(*r, *c).map(Overlap::neg)
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AxisProj {
    min: f32,
    max: f32,
}

impl AxisProj {
    fn project_rotated_rect(edge: Vector2<f32>, r: RotatedRect) -> Self {
        use std::cmp::Ordering::Equal;

        Self {
            min: r
                .corners()
                .iter()
                .map(|p| edge.dot(&p.coords))
                .min_by(|d1, d2| d1.partial_cmp(d2).unwrap_or(Equal))
                .unwrap(),
            max: r
                .corners()
                .iter()
                .map(|p| edge.dot(&p.coords))
                .max_by(|d1, d2| d1.partial_cmp(d2).unwrap_or(Equal))
                .unwrap(),
        }
    }

    fn interval_distance(self, other: AxisProj) -> f32 {
        // Calculate distance between two intervals, returning negative values
        // if the intervals overlap.
        if self.min < other.min {
            other.min - self.max
        } else {
            self.min - other.max
        }
    }
}
