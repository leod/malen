use nalgebra::{Point3, Vector2, Vector3};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};
use crevice::{glsl::GlslStruct, std140::AsStd140};

use crate::{
    attributes,
    geom::Rect,
    gl::{Attribute, UniformBlock, Vertex},
    Color3,
};

#[derive(Debug, Clone)]
pub struct GlobalLightParams {
    pub ambient: Color3,
    pub gamma: f32,
    pub front_glow: f32,
    pub back_glow: f32,
    pub angle_fall_off_size: f32,
    pub angle_fall_off_factor: f32,
}

impl Default for GlobalLightParams {
    fn default() -> Self {
        Self {
            ambient: Color3::from_u8(0, 0, 0),
            gamma: 2.2,
            front_glow: 10.0,
            back_glow: 40.0,
            angle_fall_off_size: std::f32::consts::PI / 20.0,
            angle_fall_off_factor: 10.0,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct GlobalLightParamsBlock {
    pub ambient: Vector3<f32>,
    pub gamma: f32,
    pub front_glow: f32,
    pub back_glow: f32,
    pub angle_fall_off_size: f32,
    pub angle_fall_off_factor: f32,
}

impl UniformBlock for GlobalLightParamsBlock {}

impl From<GlobalLightParams> for GlobalLightParamsBlock {
    fn from(params: GlobalLightParams) -> Self {
        GlobalLightParamsBlock {
            ambient: Vector3::new(params.ambient.r, params.ambient.g, params.ambient.b),
            gamma: params.gamma,
            front_glow: params.front_glow,
            back_glow: params.back_glow,
            angle_fall_off_size: params.angle_fall_off_size,
            angle_fall_off_factor: params.angle_fall_off_factor,
        }
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Light {
    pub position: Point3<f32>,
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
            center: self.position.xy(),
            size: 2.0 * self.radius * Vector2::new(1.0, 1.0),
        }
    }
}
