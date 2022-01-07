use nalgebra::Point3;

use crate::{
    gl::{PrimitiveMode, Vertex},
    math::Line,
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

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>);
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
    pub line: Line,
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

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

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

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

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

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });
        }
    }
}

impl Geometry<LineTag> for ColorRect {
    type Vertex = ColorVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_line_indices(vertices.len() as u32));

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

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });
        }
    }
}

impl Geometry<LineTag> for ColorLine {
    type Vertex = ColorVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.push(vertices.len() as u32);
        elements.push(vertices.len() as u32 + 1);

        vertices.push(ColorVertex {
            position: Point3::new(self.line.0.x, self.line.0.y, self.z),
            color: self.color,
        });
        vertices.push(ColorVertex {
            position: Point3::new(self.line.1.x, self.line.1.y, self.z),
            color: self.color,
        });
    }
}

pub fn quad_triangle_indices(start_index: u32) -> [u32; 6] {
    [
        start_index,
        start_index + 1,
        start_index + 2,
        start_index + 2,
        start_index + 3,
        start_index,
    ]
}

pub fn quad_line_indices(start_index: u32) -> [u32; 8] {
    [
        start_index + 0,
        start_index + 1,
        start_index + 1,
        start_index + 2,
        start_index + 2,
        start_index + 3,
        start_index + 3,
        start_index + 0,
    ]
}
