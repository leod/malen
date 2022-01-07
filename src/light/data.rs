use nalgebra::{Point2, Vector2, Vector3};

use bytemuck::Zeroable;
use bytemuck_derive::{Pod, Zeroable};
use crevice::{glsl::GlslStruct, std140::AsStd140};

use crate::{
    attributes,
    data::{quad_triangle_indices, Geometry, LineTag, TriangleTag},
    gl::{Attribute, UniformBlock, Vertex},
    math::Line,
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
pub struct OccluderLineVertex {
    pub line_0: Point2<f32>,
    pub line_1: Point2<f32>,
    pub order: i32,
    pub ignore_light_index: i32,
}

impl Vertex for OccluderLineVertex {
    fn attributes() -> Vec<Attribute> {
        attributes!["a_": line_0, line_1, order, ignore_light_index]
    }
}

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct LightAreaVertex {
    pub position: Point2<f32>,
    pub light_index: i32,
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
pub struct OccluderLine {
    pub line: Line,
    pub ignore_light_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct OccluderRect {
    pub rect: Rect,
    pub ignore_light_index: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct LightRect {
    pub light_index: i32,
    pub light: Light,
    pub rect: Rect,
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
}

impl LightInstance {
    pub fn from_light(light: Light) -> Self {
        Self {
            position: light.position,
            radius: light.radius,
        }
    }
}

impl From<Line> for OccluderLine {
    fn from(line: Line) -> Self {
        OccluderLine {
            line,
            ignore_light_index: None,
        }
    }
}

impl From<Rect> for OccluderRect {
    fn from(rect: Rect) -> Self {
        OccluderRect {
            rect,
            ignore_light_index: None,
        }
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

impl Geometry<LineTag> for OccluderRect {
    type Vertex = OccluderLineVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        for line in self
            .rect
            .lines()
            .iter()
            .chain(self.rect.caps().iter())
            .copied()
        {
            OccluderLine {
                line,
                ignore_light_index: self.ignore_light_index,
            }
            .write(elements, vertices);
        }
    }
}

impl Geometry<TriangleTag> for LightRect {
    type Vertex = LightAreaVertex;

    fn write(&self, elements: &mut Vec<u32>, vertices: &mut Vec<Self::Vertex>) {
        elements.extend_from_slice(&quad_triangle_indices(elements.len() as u32));

        for p in self.rect.corners() {
            vertices.push(LightAreaVertex {
                position: p,
                light_index: self.light_index,
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
