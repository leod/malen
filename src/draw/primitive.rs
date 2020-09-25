use golem::{Attribute, AttributeType, Dimension};

use crate::{Color, Matrix3, Point2, Point3, Vector2};

pub use golem::GeometryMode;

pub trait Vertex {
    fn attributes() -> Vec<Attribute>;
    fn num_values() -> usize;
    fn append(&self, out: &mut Vec<f32>);
}

pub struct ColorVertex {
    /// The vertex position in world coordinates.
    ///
    /// We apply the model transformation on CPU. This seems to be the
    /// easiest way to render moderate amounts of sprites in a somewhat
    /// performant way with WebGL 1: We don't have an easy way to send the
    /// per-sprite data to GPU, since we don't have access to UBOs and SSBOs.
    pub world_pos: Point3,
    pub color: Color,
}

impl Vertex for ColorVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn num_values() -> usize {
        3 + 4
    }

    fn append(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.world_pos.z,
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
        ]);
    }
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub corners: [Point2; 4],
    pub z: f32,
}

impl Quad {
    pub fn new(transform: &Matrix3) -> Self {
        Self {
            corners: [
                (transform * Point3::new(-0.5, -0.5, 1.0)).xy(),
                (transform * Point3::new(-0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, -0.5, 1.0)).xy(),
            ],
            z: transform[(2, 2)],
        }
    }

    pub fn axis_aligned(pos: Point3, size: Vector2) -> Self {
        Self {
            corners: [
                // Top left
                pos.xy() + Vector2::new(-0.5, -0.5).component_mul(&size),
                // Bottom left
                pos.xy() + Vector2::new(-0.5, 0.5).component_mul(&size),
                // Bottom right
                pos.xy() + Vector2::new(0.5, 0.5).component_mul(&size),
                // Top right
                pos.xy() + Vector2::new(0.5, -0.5).component_mul(&size),
            ],
            z: pos.z,
        }
    }
}
