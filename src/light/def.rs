use nalgebra::{Point2, Vector2, Vector3, Vector4};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};
use crevice::{glsl::GlslStruct, std140::AsStd140};

use crate::{
    attributes,
    geom::Rect,
    gl::{Attribute, UniformBlock, Vertex},
    Color3,
};

use super::light_area::LightAreaVertex;

#[derive(Debug, Clone)]
pub struct GlobalLightParams {
    pub ambient: Color3,
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct GlobalLightParamsBlock {
    pub ambient: Vector3<f32>,
}

impl UniformBlock for GlobalLightParamsBlock {}

impl From<GlobalLightParams> for GlobalLightParamsBlock {
    fn from(params: GlobalLightParams) -> Self {
        GlobalLightParamsBlock {
            ambient: Vector3::new(params.ambient.r, params.ambient.g, params.ambient.b),
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Light {
    pub position: Point2<f32>,
    pub radius: f32,
    pub angle: f32,
    pub angle_size: f32,
    pub start: f32,
    pub color: Color3,
}

impl Vertex for Light {
    fn attributes() -> Vec<Attribute> {
        attributes![
            "i_light_": position,
            radius,
            angle,
            angle_size,
            start,
            color
        ]
    }
}

impl Light {
    pub fn rect(&self) -> Rect {
        Rect {
            center: self.position,
            size: 2.0 * self.radius * Vector2::new(1.0, 1.0),
        }
    }

    pub fn to_light_vertex(&self, position: Point2<f32>, light_index: i32) -> LightAreaVertex {
        LightAreaVertex {
            position,
            light_index: light_index as f32,
            light_position: self.position,
            light_params: Vector4::new(self.radius, self.angle, self.angle_size, self.start),
            light_color: self.color,
        }
    }
}
