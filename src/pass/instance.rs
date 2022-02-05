use bytemuck::{Pod, Zeroable};
use nalgebra::{Point2, Vector2};

use crate::{
    attributes,
    gl::{Attribute, Vertex},
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
        attributes![position, z, angle, scale, color]
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
