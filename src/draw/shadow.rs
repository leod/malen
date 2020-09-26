use golem::{
    Attribute, AttributeType, ColorFormat, Dimension, GeometryMode, NumberType, ShaderDescription,
    ShaderProgram, Surface, Texture, TextureFilter, TextureWrap, Uniform, UniformType,
    UniformValue,
};

use crate::{
    draw::{Batch, Vertex},
    Context, Error, Point2,
};

pub struct LineSegment {
    pub world_pos_p: Point2,
    pub world_pos_q: Point2,
    pub order: f32,
}

impl Vertex for LineSegment {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos_p", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_world_pos_q", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_order", AttributeType::Scalar),
        ]
    }

    fn num_values() -> usize {
        2 * 2 + 1
    }

    fn append(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos_p.x,
            self.world_pos_p.y,
            self.world_pos_q.x,
            self.world_pos_q.y,
            self.order,
        ])
    }
}

pub struct Light {
    pub world_pos: Point2,
}

pub struct ShadowMap {
    max_num_lights: usize,
    shadow_surface: Surface,
    shadow_shader: ShaderProgram,
}

impl ShadowMap {
    pub fn new(ctx: &Context, resolution: usize, max_num_lights: usize) -> Result<Self, Error> {
        let mut shadow_texture = Texture::new(ctx.golem_context())?;
        shadow_texture.set_image(
            None,
            resolution as u32,
            max_num_lights as u32,
            ColorFormat::RGB,
        );
        shadow_texture.set_magnification(TextureFilter::Nearest)?;
        shadow_texture.set_minification(TextureFilter::Nearest)?;
        shadow_texture.set_wrap_h(TextureWrap::ClampToEdge)?;
        shadow_texture.set_wrap_v(TextureWrap::ClampToEdge)?;

        let shadow_surface = Surface::new(ctx.golem_context(), shadow_texture)?;

        let shadow_shader = ShaderProgram::new(
            ctx.golem_context(),
            ShaderDescription {
                vertex_input: &LineSegment::attributes(),
                fragment_input: &[
                    Attribute::new("v_edge", AttributeType::Vector(Dimension::D4)),
                    Attribute::new("v_angle", AttributeType::Scalar),
                ],
                uniforms: &[Uniform::new(
                    "light_world_pos",
                    UniformType::Vector(NumberType::Float, Dimension::D2),
                )],
                vertex_shader: r#"
                float angle_to_light(vec2 world_pos) {
                    vec2 delta = world_pos - light_world_pos;
                    return atan(delta.y, delta.x);
                }

                void main() {
                    float angle_p = angle_to_light(a_world_pos_p);
                    float angle_q = angle_to_light(a_world_pos_q);

                    v_edge = vec4(a_world_pos_p, a_world_pos_q);
                    v_edge = mix(v_edge, v_edge.zwxy, step(angle_p, angle_q));
                    v_angle = angle_p;

                    gl_Position = vec4(
                        angle_p / 3.141592,
                        0.0,
                        0.0,
                        1.0
                    );
                }
                "#,
                fragment_shader: r#"
                void main() {
                    gl_FragColor = vec4(v_angle, v_angle, v_angle, v_angle);
                }
                "#,
            },
        )?;

        Ok(Self {
            max_num_lights,
            shadow_surface,
            shadow_shader,
        })
    }

    pub fn draw_occluder_batch(
        &mut self,
        ctx: &Context,
        batch: &mut Batch<LineSegment>,
        lights: &[Light],
    ) -> Result<(), Error> {
        assert!(batch.geometry_mode() == GeometryMode::Lines);
        assert!(
            lights.len() <= self.max_num_lights,
            "Too many lights in ShadowMap::draw_occluder_batch: Got {} vs. max_num_lights {}",
            lights.len(),
            self.max_num_lights,
        );

        batch.flush();

        self.shadow_surface.bind();

        for (light_idx, light) in lights.iter().enumerate() {
            self.shadow_shader.bind();
            self.shadow_shader.set_uniform(
                "light_world_pos",
                UniformValue::Vector2(light.world_pos.coords.into()),
            )?;

            unsafe {
                self.shadow_shader.draw(
                    &batch.buffers().vertices,
                    &batch.buffers().elements,
                    0..batch.buffers().num_elements,
                    GeometryMode::Lines,
                )?;
            }
        }

        Surface::unbind(ctx.golem_context());

        Ok(())
    }
}
