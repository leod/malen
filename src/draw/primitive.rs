use golem::{Attribute, AttributeType, Dimension};
use nalgebra::{Matrix3, Point2, Point3, Vector2};

use crate::{AaRect, Color4};

pub use golem::GeometryMode;

pub trait Vertex {
    fn attributes() -> Vec<Attribute>;
    fn write(&self, out: &mut Vec<f32>);
}

pub trait Geometry {
    type Vertex: Vertex;

    fn mode() -> GeometryMode;
}

#[derive(Debug, Clone)]
pub struct Line<V> {
    pub points: [V; 2],
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ColVertex {
    /// The vertex position in world coordinates.
    ///
    /// We apply the model transformation on CPU. This seems to be the
    /// easiest way to render moderate amounts of sprites in a somewhat
    /// performant way with WebGL 1: We don't have an easy way to send the
    /// per-sprite data to GPU, since we don't have access to UBOs and SSBOs.
    pub world_pos: Point3<f32>,
    pub color: Color4,
}

impl Vertex for ColVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[self.world_pos.x, self.world_pos.y, self.world_pos.z]);
        self.color.write(out);
    }
}

#[derive(Debug, Clone)]
pub struct TexVertex {
    /// The vertex position in world coordinates.
    pub world_pos: Point3<f32>,
    pub tex_coords: Point2<f32>,
}

impl Vertex for TexVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_tex_coords", AttributeType::Vector(Dimension::D2)),
        ]
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

#[derive(Debug, Clone)]
pub struct TexColVertex {
    /// The vertex position in world coordinates.
    pub world_pos: Point3<f32>,
    pub tex_coords: Point2<f32>,
    pub color: Color4,
}

impl Vertex for TexColVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_tex_coords", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
        ]
    }

    fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.world_pos.z,
            self.tex_coords.x,
            self.tex_coords.y,
        ]);
        self.color.write(out);
    }
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub corners: [Point2<f32>; 4],
}

impl Quad {
    pub const TRIANGLE_INDICES: &'static [u32] = &[0, 1, 2, 2, 3, 0];

    pub fn corners() -> [Vector2<f32>; 4] {
        [
            Vector2::new(-0.5, -0.5),
            Vector2::new(-0.5, 0.5),
            Vector2::new(0.5, 0.5),
            Vector2::new(0.5, -0.5),
        ]
    }

    pub fn new(transform: &Matrix3<f32>) -> Self {
        Self {
            corners: [
                (transform * Point3::new(-0.5, -0.5, 1.0)).xy(),
                (transform * Point3::new(-0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, 0.5, 1.0)).xy(),
                (transform * Point3::new(0.5, -0.5, 1.0)).xy(),
            ],
        }
    }

    pub fn axis_aligned(center: Point2<f32>, size: Vector2<f32>) -> Self {
        Self {
            corners: [
                // Top left
                center + Self::corners()[0].component_mul(&size),
                // Bottom left
                center + Self::corners()[1].component_mul(&size),
                // Bottom right
                center + Self::corners()[2].component_mul(&size),
                // Top right
                center + Self::corners()[3].component_mul(&size),
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

impl From<AaRect> for Quad {
    fn from(rect: AaRect) -> Quad {
        Quad::axis_aligned(rect.center, rect.size)
    }
}
