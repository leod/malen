use bytemuck::{offset_of, Zeroable};
use bytemuck_derive::{Pod, Zeroable};
use nalgebra::{Point2, Vector2};

use crate::{
    gl::{attribute, Attribute, Vertex},
    Color4,
};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct ColorInstance {
    pub position: Point2<f32>,
    pub z: f32,
    pub angle: f32,
    pub scale: Vector2<f32>,
    pub color: Color4,
}

impl Vertex for ColorInstance {
    fn attributes() -> Vec<Attribute> {
        vec![
            attribute::<Point2<f32>>("i_position", offset_of!(Self::zeroed(), Self, position)),
            attribute::<f32>("i_z", offset_of!(Self::zeroed(), Self, z)),
            attribute::<f32>("i_angle", offset_of!(Self::zeroed(), Self, angle)),
            attribute::<Vector2<f32>>("i_scale", offset_of!(Self::zeroed(), Self, scale)),
            attribute::<Color4>("i_color", offset_of!(Self::zeroed(), Self, color)),
        ]
    }
}

impl Default for ColorInstance {
    fn default() -> Self {
        Self {
            position: Point2::origin(),
            z: 0.0,
            angle: 0.0,
            scale: Vector2::new(1.0, 1.0),
            color: Color4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}
