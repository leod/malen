use golem::{
    Attribute, AttributeType, Dimension, ShaderDescription, ShaderProgram, Texture, Uniform,
    UniformType, UniformValue,
};

use crate::{
    draw::{ColVertex, DrawUnit, TexColVertex, Vertex},
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

    pub fn draw(
        &mut self,
        transform: &Matrix3,
        draw_unit: &DrawUnit<ColVertex>,
    ) -> Result<(), Error> {
        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(transform)),
        )?;

        draw_unit.draw(&self.shader)
    }
}

pub struct TexColPass {
    shader: ShaderProgram,
}

impl TexColPass {
    pub fn new_golem(ctx: &golem::Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx,
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

    pub fn new(ctx: &Context) -> Result<Self, Error> {
        Self::new_golem(ctx.golem_ctx())
    }

    pub fn draw(
        &mut self,
        transform: &Matrix3,
        tex: &Texture,
        draw_unit: &DrawUnit<TexColVertex>,
    ) -> Result<(), Error> {
        tex.set_active(std::num::NonZeroU32::new(1).unwrap());

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(transform)),
        )?;
        self.shader.set_uniform("my_tex", UniformValue::Int(1))?;

        draw_unit.draw(&self.shader)?;

        // FIXME: Unbind texture

        Ok(())
    }
}
