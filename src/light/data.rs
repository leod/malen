use nalgebra::{Point2, Vector3};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};

use crate::{
    attributes,
    data::{quad_triangle_indices, Geometry, LineTag, TriangleTag},
    gl::{Attribute, Vertex},
    math::Line,
    Color3, Rect,
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
pub struct LightInstance {
    pub position: Point2<f32>,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct OccluderLineVertex {
    pub line_0: Point2<f32>,
    pub line_1: Point2<f32>,
    pub order: i32,
    pub ignore_light_index: i32,
}

#[derive(Debug, Clone)]
pub struct OccluderLine {
    pub line: Line,
    pub ignore_light_index: Option<u32>,
}

pub struct LightRect {
    pub rect: Rect,
    pub light: Light,
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LightAreaVertex {
    pub position: Point2<f32>,
    pub light_position: Point2<f32>,
    pub light_params: Vector3<f32>,
    pub light_color: Color3,
}

impl Vertex for LightInstance {
    fn attributes() -> Vec<Attribute> {
        attributes!["i_light_": position, radius]
    }
}

impl Vertex for OccluderLineVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": line_0, line_1, order, ignore_light_index]
    }
}

impl Vertex for LightAreaVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": position, light_position, light_params, light_color]
    }
}

impl Geometry<LineTag> for OccluderLine {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let ignore_light_index = self
            .ignore_light_index
            .map_or(-1, |i| i32::try_from(i).unwrap());

        let start_index = elements.len() as u32;
        elements.extend_from_slice(&[
            start_index,
            start_index + 1,
            start_index + 2,
            start_index + 3,
        ]);

        vertices.extend_from_slice(&[
            OccluderLineVertex {
                line_0: self.line.0,
                line_1: self.line.1,
                order: 0,
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 1,
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.0,
                line_1: self.line.1,
                order: 2,
                ignore_light_index,
            },
            OccluderLineVertex {
                line_0: self.line.1,
                line_1: self.line.0,
                order: 3,
                ignore_light_index,
            },
        ]);
    }
}

impl Geometry<TriangleTag> for LightRect {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(elements.len() as u32));

        for p in self.rect.corners() {
            vertices.push(LightAreaVertex {
                position: p,
                light_position: self.light.position,
                light_params: Vector3::new(
                    self.light.radius,
                    self.light.angle,
                    self.light.angle_size,
                ),
                light_color: self.light.color,
            });
        }
    }
}
