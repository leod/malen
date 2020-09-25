use golem::{
    Attribute, AttributeType, Dimension, ElementBuffer, GeometryMode, ShaderDescription,
    ShaderProgram, Uniform, UniformType, UniformValue, VertexBuffer,
};

use crate::{
    draw::{Batch, ColorVertex, Vertex},
    geom::matrix3_to_flat_array,
    Context, Error, Matrix3,
};

pub struct ColorPass {
    shader: ShaderProgram,
}

impl ColorPass {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx.golem_context(),
            ShaderDescription {
                vertex_input: &ColorVertex::attributes(),
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

        Ok(ColorPass { shader })
    }

    pub fn draw_batch(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        batch: &mut Batch<ColorVertex>,
    ) -> Result<(), Error> {
        batch.flush();

        self.draw_buffers_unchecked(
            projection,
            view,
            &batch.buffers().vertices,
            &batch.buffers().elements,
            batch.buffers().num_elements,
            batch.geometry_mode(),
        )
    }

    pub fn draw_buffers_unchecked(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        vertices: &VertexBuffer,
        elements: &ElementBuffer,
        num_elements: usize,
        geometry_mode: GeometryMode,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;

        unsafe {
            self.shader
                .draw(vertices, elements, 0..num_elements, geometry_mode)?;
        }

        Ok(())
    }
}
