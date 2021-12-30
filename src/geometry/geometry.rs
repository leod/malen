use nalgebra::Point2;

use crate::{Rect, RotatedRect, Color4, gl::{Vertex, PrimitiveMode}};

use super::SpriteVertex;

pub trait PrimitiveTag {
    fn primitive_mode() -> PrimitiveMode;
}

pub struct TriangleTag {
}

pub struct LineTag {
}

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

impl Geometry<TriangleTag> for Sprite {
    type Vertex = SpriteVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        let corners = self.rect.corners();
        let uv_corners = self.uv.corners();
    }
}