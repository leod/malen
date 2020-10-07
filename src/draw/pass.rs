use golem::{
    Attribute, AttributeType, Dimension, GeometryMode, ShaderDescription, ShaderProgram, Uniform,
    UniformType, UniformValue,
};

use crate::{
    draw::{AsBuffersSlice, Batch, BuffersSlice, ColorVertex, Vertex},
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

        Ok(Self { shader })
    }

    pub fn draw_batch(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        batch: &mut Batch<ColorVertex>,
    ) -> Result<(), Error> {
        batch.flush();

        // TODO: I believe this is safe, because Batch in its construction
        // (see Batch::push_element) makes sure that each element points to
        // a valid index in the vertex buffer. We need to verify this though.
        // We also need to verify if golem::ShaderProgram::draw has any
        // additional requirements for safety.
        unsafe {
            self.draw_buffers(
                projection,
                view,
                batch.buffers().as_buffers_slice(),
                batch.geometry_mode(),
            )
        }
    }

    pub unsafe fn draw_buffers(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        buffers: BuffersSlice<ColorVertex>,
        geometry_mode: GeometryMode,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;

        buffers.draw(&self.shader, geometry_mode)?;

        Ok(())
    }
}
