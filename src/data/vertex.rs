use nalgebra::{Point2, Point3};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    attributes,
    gl::{Attribute, Vertex},
    Color4,
};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpriteVertex {
    pub position: Point3<f32>,
    pub tex_coords: Point2<f32>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ColorSpriteVertex {
    pub position: Point3<f32>,
    pub tex_coords: Point2<f32>,
    pub color: Color4,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ColorVertex {
    pub position: Point3<f32>,
    pub color: Color4,
}

impl Vertex for SpriteVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": position, tex_coords]
    }
}

impl Vertex for ColorSpriteVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": position, tex_coords, color]
    }
}

impl Vertex for ColorVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": position, color]
    }
}
