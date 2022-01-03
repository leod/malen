use nalgebra::{Point2, Point3};

use crate::{
    gl::{PrimitiveMode, Vertex},
    Color4, Rect, RotatedRect,
};

use super::{ColorSpriteVertex, ColorVertex, SpriteVertex};

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
    pub rect: Rect,
    pub z: f32,
    pub tex_rect: Rect,
}

#[derive(Debug, Copy, Clone)]
pub struct ColorSprite {
    pub rect: Rect,
    pub z: f32,
    pub tex_rect: Rect,
    pub color: Color4,
}

#[derive(Debug, Copy, Clone)]
pub struct ColorRect {
    pub rect: Rect,
    pub z: f32,
    pub color: Color4,
}

#[derive(Debug, Copy, Clone)]
pub struct ColorRotatedRect {
    pub rect: RotatedRect,
    pub z: f32,
    pub color: Color4,
}

#[derive(Debug, Copy, Clone)]
pub struct ColorLine {
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

fn quad_indices(start_index: u32) -> [u32; 6] {
    [
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 2,
        start_index + 3,
        start_index,
    ]
}

impl Geometry<TriangleTag> for Sprite {
    type Vertex = SpriteVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        elements.extend_from_slice(&quad_indices(vertices.len() as u32));

        for (p, tex_coords) in self.rect.corners().iter().zip(self.tex_rect.corners()) {
            vertices.push(SpriteVertex {
                position: Point3::new(p.x, p.y, self.z),
                tex_coords,
            });
        }
    }
}

impl Geometry<TriangleTag> for ColorSprite {
    type Vertex = ColorSpriteVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        elements.extend_from_slice(&quad_indices(vertices.len() as u32));

        for (p, tex_coords) in self.rect.corners().iter().zip(self.tex_rect.corners()) {
            vertices.push(ColorSpriteVertex {
                position: Point3::new(p.x, p.y, self.z),
                tex_coords,
                color: self.color,
            });
        }
    }
}

impl Geometry<TriangleTag> for ColorRect {
    type Vertex = ColorVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        elements.extend_from_slice(&quad_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });
        }
    }
}

impl Geometry<TriangleTag> for ColorRotatedRect {
    type Vertex = ColorVertex;

    fn write(&self, vertices: &mut Vec<Self::Vertex>, elements: &mut Vec<u32>) {
        elements.extend_from_slice(&quad_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });
        }
    }
}
