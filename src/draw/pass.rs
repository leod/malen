use golem::{
    Attribute, AttributeType, Dimension, GeometryMode, ShaderDescription, ShaderProgram, Texture,
    Uniform, UniformType, UniformValue,
};

use crate::{
    draw::{AsBuffersSlice, Batch, BuffersSlice, ColVertex, TexColVertex, Vertex},
    geom::matrix3_to_flat_array,
    Context, Error, Matrix3,
};

pub struct ColPass {
    shader: ShaderProgram,
}

impl ColPass {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx.golem_ctx(),
            ShaderDescription {
                vertex_input: &ColVertex::attributes(),
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
        batch: &mut Batch<ColVertex>,
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
        buffers: BuffersSlice<ColVertex>,
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

pub struct TexColPass {
    shader: ShaderProgram,
}

impl TexColPass {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx.golem_ctx(),
            ShaderDescription {
                vertex_input: &TexColVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_color", AttributeType::Vector(Dimension::D4)),
                    Attribute::new("v_tex_coords", AttributeType::Vector(Dimension::D2)),
                ],
                uniforms: &[
                    Uniform::new("mat_projection_view", UniformType::Matrix(Dimension::D3)),
                    Uniform::new("my_tex", UniformType::Sampler2D),
                ],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos.xy, 1.0);
                    gl_Position = vec4(p.xy, a_world_pos.z, 1.0);
                    v_color = a_color;
                    v_tex_coords = a_tex_coords;
                }
                "#,
                fragment_shader: r#"
                void main() {
                    gl_FragColor = v_color * texture(my_tex, v_tex_coords);
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
        texture: &Texture,
        batch: &mut Batch<TexColVertex>,
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
                texture,
                batch.buffers().as_buffers_slice(),
                batch.geometry_mode(),
            )
        }
    }

    pub unsafe fn draw_buffers(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        texture: &Texture,
        buffers: BuffersSlice<TexColVertex>,
        geometry_mode: GeometryMode,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        texture.set_active(std::num::NonZeroU32::new(1).unwrap());

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;
        self.shader.set_uniform("my_tex", UniformValue::Int(1))?;

        buffers.draw(&self.shader, geometry_mode)?;

        // FIXME: Unbind texture

        Ok(())
    }
}
