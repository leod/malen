use golem::{Attribute, AttributeType, Dimension};

use crate::{Color, Matrix3, Point2, Point3, Vector2};

pub use golem::GeometryMode;

pub trait Vertex {
    fn attributes() -> Vec<Attribute>;
    fn num_values() -> usize;
    fn write(&self, out: &mut Vec<f32>);
}

pub trait Geometry {
    type Vertex: Vertex;

    fn mode() -> GeometryMode;
}

pub struct Line<V> {
    pub points: [V; 2],
}

pub struct Triangle<V> {
    pub points: [V; 3],
}

impl<V: Vertex> Geometry for Line<V> {
    type Vertex = V;

    fn mode() -> GeometryMode {
        GeometryMode::Lines
    }
}

impl<V: Vertex> Geometry for Triangle<V> {
    type Vertex = V;

    fn mode() -> GeometryMode {
        GeometryMode::Triangles
    }
}

pub struct ColVertex {
    /// The vertex position in world coordinates.
    ///
    /// We apply the model transformation on CPU. This seems to be the
    /// easiest way to render moderate amounts of sprites in a somewhat
    /// performant way with WebGL 1: We don't have an easy way to send the
    /// per-sprite data to GPU, since we don't have access to UBOs and SSBOs.
    pub world_pos: Point3,
    pub color: Color,
}

impl Vertex for ColVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn num_values() -> usize {
        3 + 4
    }

    fn write(&self, out: &mut Vec<f32>) {
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

pub struct TexVertex {
    /// The vertex position in world coordinates.
    pub world_pos: Point3,
    pub tex_coords: Point2,
}

impl Vertex for TexVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_tex_coords", AttributeType::Vector(Dimension::D2)),
        ]
    }

    fn num_values() -> usize {
        3 + 2
    }

    fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.world_pos.z,
            self.tex_coords.x,
            self.tex_coords.y,
        ]);
    }
}

pub struct TexColVertex {
    /// The vertex position in world coordinates.
    pub world_pos: Point3,
    pub tex_coords: Point2,
    pub color: Color,
}

impl Vertex for TexColVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_tex_coords", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn num_values() -> usize {
        3 + 2 + 4
    }

    fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.world_pos.z,
            self.tex_coords.x,
            self.tex_coords.y,
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
}

impl Quad {
    pub const TRIANGLE_INDICES: &'static [u32] = &[0, 1, 2, 2, 3, 0];

    pub fn corners() -> [Vector2; 4] {
        [
            Vector2::new(-0.5, -0.5),
            Vector2::new(-0.5, 0.5),
            Vector2::new(0.5, 0.5),
            Vector2::new(0.5, -0.5),
        ]
    }

    pub fn new(transform: &Matrix3) -> Self {
        Self {
            corners: [
                (transform * Point3::new(-0.5, -0.5, 1.0)).xy(),
                (transform * Point3::new(-0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, -0.5, 1.0)).xy(),
            ],
        }
    }

    pub fn axis_aligned(pos: Point2, size: Vector2) -> Self {
        Self {
            corners: [
                // Top left
                pos + Self::corners()[0].component_mul(&size),
                // Bottom left
                pos + Self::corners()[1].component_mul(&size),
                // Bottom right
                pos + Self::corners()[2].component_mul(&size),
                // Top right
                pos + Self::corners()[3].component_mul(&size),
            ],
        }
    }

    pub fn triangle_indices(first_idx: u32) -> [u32; 6] {
        [
            first_idx + Self::TRIANGLE_INDICES[0],
            first_idx + Self::TRIANGLE_INDICES[1],
            first_idx + Self::TRIANGLE_INDICES[2],
            first_idx + Self::TRIANGLE_INDICES[3],
            first_idx + Self::TRIANGLE_INDICES[4],
            first_idx + Self::TRIANGLE_INDICES[5],
        ]
    }
}
