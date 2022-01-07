use nalgebra::{Point2, Vector2, Vector3};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};
use crevice::{glsl::GlslStruct, std140::AsStd140};

use crate::{
    attributes,
    data::{quad_triangle_indices, Geometry, TriangleTag},
    gl::{Attribute, UniformBlock, Vertex},
    Color3, Rect,
};

#[derive(Debug, Clone)]
pub struct GlobalLightParams {
    pub ambient: Color3,
}

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct GlobalLightParamsBlock {
    pub ambient: Vector3<f32>,
}

impl UniformBlock for GlobalLightParamsBlock {}

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
pub struct LightInstance {
    pub position: Point2<f32>,
    pub radius: f32,
}

impl Vertex for LightInstance {
    fn attributes() -> Vec<Attribute> {
        attributes!["i_light_": position, radius]
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LightAreaVertex {
    pub position: Point2<f32>,
    pub light_index: f32,
    pub light_position: Point2<f32>,
    pub light_params: Vector3<f32>,
    pub light_color: Color3,
}

impl Vertex for LightAreaVertex {
    fn attributes() -> Vec<Attribute> {
        attributes![
            "a_": position,
            light_index,
            light_position,
            light_params,
            light_color
        ]
    }
}

#[derive(Debug, Clone)]
pub struct LightRect {
    pub light_index: i32,
    pub light: Light,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct LightCircleSegment {
    pub light_index: i32,
    pub light: Light,
    pub num_segments: usize,
}

impl From<GlobalLightParams> for GlobalLightParamsBlock {
    fn from(params: GlobalLightParams) -> Self {
        GlobalLightParamsBlock {
            ambient: Vector3::new(params.ambient.r, params.ambient.g, params.ambient.b),
        }
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
            light_params: Vector3::new(self.radius, self.angle, self.angle_size),
            light_color: self.color,
        }
    }
}

impl LightInstance {
    pub fn from_light(light: Light) -> Self {
        Self {
            position: light.position,
            radius: light.radius,
        }
    }
}

impl Geometry<TriangleTag> for LightRect {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(self.light.to_light_vertex(p, self.light_index));
        }
    }
}

impl Geometry<TriangleTag> for LightCircleSegment {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let start_index = vertices.len() as u32;

        vertices.push(
            self.light
                .to_light_vertex(self.light.position, self.light_index),
        );

        for i in 0..=self.num_segments {
            let angle = self.light.angle - self.light.angle_size / 2.0
                + i as f32 / self.num_segments as f32 * self.light.angle_size;
            let p =
                self.light.position + Vector2::new(angle.cos(), angle.sin()) * self.light.radius;
            vertices.push(self.light.to_light_vertex(p, self.light_index));

            if i != self.num_segments {
                elements.extend_from_slice(&[
                    start_index,
                    start_index + 1 + ((i + 1) % (self.num_segments + 1)) as u32,
                    start_index + 1 + i as u32,
                ]);
            }
        }
    }
}
