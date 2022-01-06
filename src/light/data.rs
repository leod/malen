use nalgebra::Point2;

use bytemuck::{offset_of, Zeroable};
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    attributes,
    gl::{Attribute, Vertex},
    Color3,
};

#[derive(Debug, Clone)]
pub struct Light {
    pub position: Point2<f32>,
    pub radius: f32,
    pub angle: f32,
    pub angle_size: f32,
    pub color: Color3,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct OccluderLineVertex {
    pub line_a: Point2<f32>,
    pub line_b: Point2<f32>,
    pub order: i32,
    pub ignore_light_index: i32,
}

impl Vertex for OccluderLineVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": line_a, line_b, order, ignore_light_index]
    }
}
