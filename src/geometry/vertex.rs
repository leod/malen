use nalgebra::{Point2, Point3};

use bytemuck::{offset_of, Zeroable};
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    gl::{attribute, Attribute, Vertex},
    Color4,
};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct SpriteVertex {
    pub position: Point3<f32>,
    pub uv: Point2<f32>,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredSpriteVertex {
    pub position: Point3<f32>,
    pub uv: Point2<f32>,
    pub color: Color4,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex {
    pub position: Point3<f32>,
    pub color: Color4,
}

impl Vertex for SpriteVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            attribute::<Point3<f32>>("v_position", offset_of!(Self::zeroed(), Self, position)),
            attribute::<Point2<f32>>("v_uv", offset_of!(Self::zeroed(), Self, uv)),
        ]
    }
}

impl Vertex for ColoredSpriteVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            attribute::<Point3<f32>>("v_position", offset_of!(Self::zeroed(), Self, position)),
            attribute::<Point2<f32>>("v_uv", offset_of!(Self::zeroed(), Self, uv)),
            attribute::<Color4>("v_color", offset_of!(Self::zeroed(), Self, color)),
        ]
    }
}

impl Vertex for ColoredVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            attribute::<Point3<f32>>("v_position", offset_of!(Self::zeroed(), Self, position)),
            attribute::<Color4>("v_color", offset_of!(Self::zeroed(), Self, color)),
        ]
    }
}
