use nalgebra::Point3;

use crate::{
    geom::{Circle, Line, Rect, RotatedRect},
    gl::{PrimitiveMode, Vertex},
    Color4,
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

pub struct ColorRotatedSprite {
    pub rect: RotatedRect,
    pub z: f32,
    pub tex_rect: Rect,
    pub color: Color4,
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

impl Geometry<TriangleTag> for ColorRotatedSprite {
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

impl Geometry<LineTag> for ColorRotatedRect {
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

pub struct ColorCircle {
    pub circle: Circle,
    pub z: f32,
    pub angle: f32,
    pub num_segments: usize,
    pub color: Color4,
}

impl Geometry<TriangleTag> for ColorCircle {
    type Vertex = ColorVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let start_index = vertices.len() as u32;

        vertices.push(ColorVertex {
            position: Point3::new(self.circle.center.x, self.circle.center.y, self.z),
            color: self.color,
        });

        for (i, p) in self
            .circle
            .points(self.angle, self.num_segments)
            .enumerate()
        {
            vertices.push(ColorVertex {
                position: Point3::new(p.x, p.y, self.z),
                color: self.color,
            });

            elements.extend_from_slice(&[
                start_index,
                start_index + 1 + ((i + 1) % self.num_segments) as u32,
                start_index + 1 + i as u32,
            ]);
        }
    }
}

impl Geometry<LineTag> for ColorCircle {
    type Vertex = ColorVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        // TODO: Alloc in ColorCircle::write
        let points = self
            .circle
            .points(self.angle, self.num_segments)
            .collect::<Vec<_>>();

        for i in 0..points.len() {
            ColorLine {
                line: Line(points[i], points[(i + 1) % points.len()]),
                z: self.z,
                color: self.color,
            }
            .write(elements, vertices);
        }
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
        start_index,
        start_index + 1,
        start_index + 1,
        start_index + 2,
        start_index + 2,
        start_index + 3,
        start_index + 3,
        start_index,
    ]
}
