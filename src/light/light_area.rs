use nalgebra::{Point2, Point4, Vector2, Vector4};

use bytemuck::{Pod, Zeroable};

use crate::{
    attributes,
    data::{quad_triangle_indices, Geometry, TriangleTag},
    geom::Rect,
    gl::{Attribute, Vertex},
    Color3,
};

use super::Light;

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LightAreaVertex {
    pub position: Point2<f32>,
    pub light_index: f32,
    pub light_position: Point4<f32>,
    pub light_params: Vector4<f32>,
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

impl LightAreaVertex {
    pub fn from_light(light_index: i32, light: &Light, position: Point2<f32>) -> Self {
        Self {
            position,
            light_index: light_index as f32,
            light_position: Point4::new(
                light.position.x,
                light.position.y,
                light.position.z,
                light.radius,
            ),
            light_params: Vector4::new(light.angle, light.angle_size, light.start, light.back_glow),
            light_color: light.color,
        }
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

impl Geometry<TriangleTag> for LightRect {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(vertices.len() as u32));

        for p in self.rect.corners() {
            vertices.push(LightAreaVertex::from_light(
                self.light_index,
                &self.light,
                p,
            ));
        }
    }
}

impl Geometry<TriangleTag> for LightCircleSegment {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        let start_index = vertices.len() as u32;

        vertices.push(LightAreaVertex::from_light(
            self.light_index,
            &self.light,
            self.light.position.xy(),
        ));

        for i in 0..=self.num_segments {
            let angle = self.light.angle - self.light.angle_size / 2.0
                + i as f32 / self.num_segments as f32 * self.light.angle_size;
            let p = self.light.position.xy()
                + Vector2::new(angle.cos(), angle.sin()) * self.light.radius;
            vertices.push(LightAreaVertex::from_light(
                self.light_index,
                &self.light,
                p,
            ));

            if i != self.num_segments {
                elements.extend_from_slice(&[
                    start_index,
                    start_index + 1 + ((i + 1) % (self.num_segments + 1)) as u32,
                    start_index + 1 + i as u32,
                ]);
            }
        }

        // TODO: Improve LightCircleSegment geometry
        if (self.light.angle_size - std::f32::consts::PI * 2.0).abs() < 0.001 {
            elements.extend_from_slice(&[
                start_index,
                start_index + 1 + self.num_segments as u32,
                start_index + 1,
            ]);
        }
    }
}
