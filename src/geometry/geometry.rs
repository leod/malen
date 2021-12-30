use nalgebra::{Point2, Point3};

use crate::{
    gl::{PrimitiveMode, Vertex},
    Color4, Rect, RotatedRect,
};

use super::SpriteVertex;

pub trait PrimitiveTag {
    fn primitive_mode() -> PrimitiveMode;
}

pub struct TriangleTag {}

pub struct LineTag {}

pub trait Geometry<Tag: PrimitiveTag> {
    type Vertex: Vertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>);
}

#[derive(Debug, Copy, Clone)]
pub struct Sprite {
    pub rect: RotatedRect,
    pub z: f32,
    pub uv: Rect,
}

#[derive(Debug, Copy, Clone)]
pub struct ColoredSprite {
    pub rect: RotatedRect,
    pub z: f32,
    pub uv: Rect,
    pub color: Color4,
}

#[derive(Debug, Copy, Clone)]
pub struct ColoredRotatedRect {
    pub rect: RotatedRect,
    pub z: f32,
    pub color: Color4,
}

#[derive(Debug, Copy, Clone)]
pub struct ColoredLine {
    pub a: Point2<f32>,
    pub b: Point2<f32>,
    pub z: f32,
    pub color: Color4,
}

impl PrimitiveTag for TriangleTag {
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Triangle
    }
}

impl PrimitiveTag for LineTag {
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Line
    }
}

fn triangle_indices(start_index: u32) -> [u32; 6] {
    [
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 2,
        start_index + 3,
        start_index + 0,
    ]
}

impl Geometry<TriangleTag> for Sprite {
    type Vertex = SpriteVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        elements.extend_from_slice(&triangle_indices(vertices.len() as u32));

        for (p, uv) in self.rect.corners().iter().zip(self.uv.corners()) {
            vertices.push(SpriteVertex {
                position: Point3::new(p.x, p.y, self.z),
                uv,
            });
        }
    }
}
