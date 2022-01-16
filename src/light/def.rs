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
    pub back_glow: f32,
    pub angle_fall_off_size: f32,
    pub angle_fall_off_factor: f32,
    pub indirect_color_scale: f32,
    pub indirect_start: f32,
    pub indirect_step_factor: f32,
    pub indirect_z: f32,
}

impl Default for GlobalLightParams {
    fn default() -> Self {
        Self {
            ambient: Color3::from_u8(0, 0, 0),
            gamma: 2.2,
            back_glow: 25.0,
            angle_fall_off_size: std::f32::consts::PI / 20.0,
            angle_fall_off_factor: 10.0,
            indirect_color_scale: 75.0,
            indirect_start: 2.0,
            indirect_step_factor: 0.65,
            indirect_z: 0.5,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct GlobalLightParamsBlock {
    pub screen_size: Vector2<f32>,
    pub ambient: Vector3<f32>,
    pub gamma: f32,
    pub back_glow: f32,
    pub angle_fall_off_size: f32,
    pub angle_fall_off_factor: f32,
    pub indirect_color_scale: f32,
    pub indirect_start: f32,
    pub indirect_step_factor: f32,
    pub indirect_z: f32,
}

impl UniformBlock for GlobalLightParamsBlock {}

impl GlobalLightParamsBlock {
    pub fn new(screen_size: Vector2<f32>, params: GlobalLightParams) -> Self {
        GlobalLightParamsBlock {
            screen_size,
            ambient: Vector3::new(params.ambient.r, params.ambient.g, params.ambient.b),
            gamma: params.gamma,
            back_glow: params.back_glow,
            angle_fall_off_size: params.angle_fall_off_size,
            angle_fall_off_factor: params.angle_fall_off_factor,
            indirect_color_scale: params.indirect_color_scale,
            indirect_start: params.indirect_start,
            indirect_step_factor: params.indirect_step_factor,
            indirect_z: params.indirect_z,
        }
    }
}

impl UniformBlock for ObjectLightParams {}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct ObjectLightParams {
    pub ambient_scale: f32,
    pub occlusion: f32,
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
