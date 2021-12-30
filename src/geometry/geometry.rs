use nalgebra::Point2;

use crate::{Rect, RotatedRect, Color4, gl::{Vertex, PrimitiveMode}};

pub trait PrimitiveTag {
    fn primitive_mode() -> PrimitiveMode;
}

pub struct TriangleTag {
}

impl TriangleTag {
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Triangle
    }
}

pub struct LineTag {
}

impl PrimitiveTag for LineTag {
    fn primitive_mode() -> PrimitiveMode {
        PrimitiveMode::Line
    }
}

pub trait Geometry<Tag: PrimitiveTag> {
    type Vertex: Vertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>);
}

pub struct Sprite {
    pub rect: RotatedRect,
    pub z: f32,
    pub uv: Rect,
}

pub struct ColSprite {
    pub rect: RotatedRect,
    pub z: f32,
    pub uv: Rect,
    pub color: Color4,
}

pub struct ColRect {
   pub rect: RotatedRect,
   pub z: f32,
   pub color: Color4,
}

pub struct ColLine {
    pub a: Point2<f32>,
    pub b: Point2<f32>,
    pub z: f32,
    pub color: Color4,
}