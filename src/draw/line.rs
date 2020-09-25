use golem::{
    Attribute, AttributeType, Dimension, ElementBuffer, GeometryMode, ShaderDescription,
    ShaderProgram, Uniform, UniformType, UniformValue, VertexBuffer,
};

use crate::{Matrix3, Point2, Color, draw::Quad, Error, geom::matrix3_to_flat_array};

pub struct ColorLine {
    pub a: Point2,
    pub b: Point2,
    pub z: f32,
    pub color: Color,
}

#[derive(Default)]
pub struct LineList {
    vertices: Vec<f32>,
    elements: Vec<u32>,
}

impl LineList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push(&mut self, line: &ColorLine) {
        let first_idx = self.vertices.len() as u32;

        self.vertices.reserve(2 * (3 + 4));

        for p in &[line.a, line.b] {
            self.vertices.extend_from_slice(&[
                p.x,
                p.y,
                line.z,
                line.color.x,
                line.color.y,
                line.color.z,
                line.color.w,
            ]);
        }

        self.elements.reserve(2);
        self.elements.push(first_idx);
        self.elements.push(first_idx + 1);
    }

    pub fn push_quad_outline(&mut self, quad: &Quad, color: Color) {
        let first_idx = self.vertices.len() as u32;

        self.vertices.reserve(4 * (3 + 4));

        for p in &quad.corners {
            self.vertices.extend_from_slice(&[
                p.x,
                p.y,
                quad.z,
                color.x,
                color.y,
                color.z,
                color.w,
            ])
        }

        self.elements.reserve(2 * 4);

        for i in &[0, 1, 1, 2, 2, 3, 3, 0] {
            self.elements.push(first_idx + i);
        }
    }
}
pub struct LineBatch {
    vertices: VertexBuffer,
    elements: ElementBuffer,
    num_elements: usize,
}

impl LineBatch {
    pub fn new(ctx: &golem::Context) -> Result<Self, Error> {
        Ok(Self {
            vertices: VertexBuffer::new(ctx)?,
            elements: ElementBuffer::new(ctx)?,
            num_elements: 0,
        })
    }

    pub fn from_list(ctx: &golem::Context, data: &LineList) -> Result<Self, Error> {
        let mut result = Self::new(ctx)?;
        result.set_data(data);
        Ok(result)
    }

    pub fn set_data(&mut self, data: &LineList) {
        self.vertices.set_data(&data.vertices);
        self.elements.set_data(&data.elements);

        self.num_elements = data.elements.len();
    }
}

pub struct LinePass {
    shader: ShaderProgram,
}

impl LinePass {
    pub fn new(ctx: &golem::Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx,
            ShaderDescription {
                vertex_input: &[
                    Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D3)),
                    Attribute::new("a_color", AttributeType::Vector(Dimension::D4)),
                ],
                fragment_input: &[Attribute::new(
                    "v_color",
                    AttributeType::Vector(Dimension::D4),
                )],
                uniforms: &[Uniform::new(
                    "mat_projection_view",
                    UniformType::Matrix(Dimension::D3),
                )],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos.xy, 1.0);
                    gl_Position = vec4(p.xy, a_world_pos.z, 1.0);
                    v_color = a_color;
                }
                "#,
                fragment_shader: r#"
                void main() {
                    gl_FragColor = v_color;
                }
                "#,
            },
        )?;

        Ok(LinePass {
            shader,
        })
    }

    pub fn draw(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        batch: &LineBatch,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;

        unsafe {
            self.shader.draw(
                &batch.vertices,
                &batch.elements,
                0..batch.num_elements,
                GeometryMode::Lines,
            )?;
        }

        Ok(())
    }
}
