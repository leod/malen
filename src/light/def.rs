use nalgebra::{Point3, Vector2, Vector3};

use bytemuck::{Pod, Zeroable};
use crevice::{glsl::GlslStruct, std140::AsStd140};

use crate::{
    attributes,
    geom::Rect,
    gl::{Attribute, UniformBlock, Vertex},
    Color3,
};

#[derive(Debug, Clone)]
pub struct LightPipelineParams {
    pub shadow_map_resolution: u32,
    pub max_num_lights: u32,
    pub num_tracing_cones: u32,
    pub num_tracing_steps: u32,
}

impl Default for LightPipelineParams {
    fn default() -> Self {
        Self {
            shadow_map_resolution: 2048,
            max_num_lights: 512,
            num_tracing_cones: 6,
            num_tracing_steps: 8,
        }
    }
}

#[derive(Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct GlobalLightProps {
    pub ambient: Vector3<f32>,
    pub gamma: f32,
    pub angle_fall_off_size: f32,
    pub angle_fall_off_factor: f32,
    pub indirect_intensity: f32,
    pub indirect_initial_offset: f32,
    pub indirect_step_factor: f32,
    pub indirect_z: f32,
    pub indirect_self_occlusion: f32,
    pub debug_mode: u32,
    pub debug_mipmap: u32,
}

impl UniformBlock for GlobalLightProps {}

impl Default for GlobalLightProps {
    fn default() -> Self {
        Self {
            ambient: Vector3::new(0.0, 0.0, 0.0),
            gamma: 2.2,
            angle_fall_off_size: std::f32::consts::PI / 20.0,
            angle_fall_off_factor: 10.0,
            indirect_intensity: 4.0,
            indirect_initial_offset: 1.0,
            indirect_step_factor: 0.4,
            indirect_z: 0.5,
            indirect_self_occlusion: 1.0,
            debug_mode: 0,
            debug_mipmap: 0,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct ObjectLightProps {
    pub occlusion: f32,
}

impl UniformBlock for ObjectLightProps {}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Light {
    pub position: Point3<f32>,
    pub radius: f32,
    pub angle: f32,
    pub angle_size: f32,
    pub start: f32,
    pub back_glow: f32,
    pub color: Color3,
}

impl Vertex for Light {
    fn attributes() -> Vec<Attribute> {
        attributes![position, radius, angle, angle_size, start, back_glow, color]
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
