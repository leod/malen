use golem::{
    blend::{BlendEquation, BlendFactor, BlendFunction, BlendMode, BlendOperation},
    Attribute, AttributeType, ColorFormat, Dimension, GeometryMode, NumberType, ShaderDescription,
    ShaderProgram, Surface, Texture, TextureFilter, TextureWrap, Uniform, UniformType,
    UniformValue,
};

use crate::{
    draw::{Batch, ColorVertex, Vertex},
    geom::matrix3_to_flat_array,
    Context, Error, Matrix3, Point2,
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
    resolution: usize,
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
            ColorFormat::RGBA,
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

                const float PI = 3.141592;

                void main() {
                    float angle_p = angle_to_light(a_world_pos_p);
                    float angle_q = angle_to_light(a_world_pos_q);

                    v_edge = vec4(a_world_pos_p, a_world_pos_q);
                    v_edge = mix(v_edge, v_edge.zwxy, step(angle_p, angle_q));

                    v_angle = angle_p;

                    if (abs(angle_p - angle_q) > PI) {
                        if (a_order == 0.0) {
                            v_angle = -PI;
                        } else if (a_order == 1.0) {
                            v_angle = min(angle_p, angle_q);
                        } else if (a_order == 2.0) {
                            v_angle = max(angle_p, angle_q);
                        } else {
                            v_angle = PI;
                        }
                    }

                    gl_Position = vec4(
                        v_angle / PI,
                        0.0,
                        0.0,
                        1.0
                    );
                }
                "#,
                fragment_shader: r#"
                float line_segment_intersection(
                    vec2 line_one_p,
                    vec2 line_one_q,
                    vec2 line_two_p,
                    vec2 line_two_q
                ) {
                    vec2 line_two_perp = vec2(
                        line_two_q.y - line_two_p.y,
                        line_two_p.x - line_two_q.x
                    );
                    float line_one_proj = dot(line_one_q - line_one_p, line_two_perp);

                    if (abs(line_one_proj) < 0.0001) {
                        return 1.0;
                    }

                    return dot(line_two_p - line_one_p, line_two_perp) / line_one_proj;
                }

                void main() {
                    float t = line_segment_intersection(
                        light_world_pos,
                        light_world_pos + vec2(cos(v_angle) * 1024.0, sin(v_angle) * 1024.0),
                        v_edge.xy,
                        v_edge.zw
                    );

                    gl_FragColor = vec4(t, t, t, t);
                }
                "#,
            },
        )?;

        Ok(Self {
            resolution,
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

        ctx.golem_context().set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Min),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

        self.shadow_surface.bind();
        ctx.golem_context()
            .set_viewport(0, 0, self.resolution as u32, self.max_num_lights as u32);
        ctx.golem_context().set_clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.golem_context().clear();

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
        ctx.golem_context().set_blend_mode(None);

        Ok(())
    }
}

pub struct ShadowedColorPass {
    shader: ShaderProgram,
}

impl ShadowedColorPass {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx.golem_context(),
            ShaderDescription {
                vertex_input: &ColorVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_color", AttributeType::Vector(Dimension::D4)),
                    Attribute::new("v_world_pos", AttributeType::Vector(Dimension::D2)),
                ],
                uniforms: &[
                    Uniform::new("mat_projection_view", UniformType::Matrix(Dimension::D3)),
                    Uniform::new("shadow_map", UniformType::Sampler2D),
                    Uniform::new(
                        "light_world_pos",
                        UniformType::Vector(NumberType::Float, Dimension::D2),
                    ),
                    Uniform::new(
                        "shadow_map_resolution",
                        UniformType::Scalar(NumberType::Float),
                    ),
                ],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos.xy, 1.0);
                    gl_Position = vec4(p.xy, a_world_pos.z, 1.0);

                    v_world_pos = a_world_pos.xy;
                    v_color = a_color;
                }
                "#,
                fragment_shader: r#"
                float angle_to_light(vec2 world_pos) {
                    vec2 delta = world_pos - light_world_pos;
                    return atan(delta.y, delta.x);
                }
                
                void main() {
                    if (gl_FragCoord.y < 10.0) {
                        gl_FragColor = vec4(texture(shadow_map, vec2(gl_FragCoord.x / 1024.0, 0.0)).rgb, 1.0);
                    } else {
                        float angle = angle_to_light(v_world_pos);
                        float dist = length(v_world_pos - light_world_pos);
                        vec2 tex_coords = vec2((angle / (2.0*3.141592)) + 0.5, 0.0);

                        float shadow_val = 0.0;
                        shadow_val += step(dist, texture(shadow_map, tex_coords).r * 1024.0) * 0.5;
                        shadow_val += step(dist, texture(shadow_map, tex_coords - vec2(2.0 / 1024.0, 0.0)).r * 1024.0) * 0.25;
                        shadow_val += step(dist, texture(shadow_map, tex_coords + vec2(2.0 / 1024.0, 0.0)).r * 1024.0) * 0.25;
                        //shadow_val /= 3.0;

                        /*if (shadow_val < 0.1 && dist > 500.0)
                            shadow_val = 1.0;*/

                        shadow_val = max(shadow_val, 0.5);

                        gl_FragColor = vec4(v_color.rgb * shadow_val, v_color.a);
                    }
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
        lights: &[Light],
        shadow_map: &ShadowMap,
        batch: &mut Batch<ColorVertex>,
    ) -> Result<(), Error> {
        batch.flush();

        let projection_view = projection * view;

        unsafe {
            shadow_map
                .shadow_surface
                .borrow_texture()
                .unwrap()
                .set_active(std::num::NonZeroU32::new(1).unwrap());
        }

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;
        self.shader.set_uniform(
            "light_world_pos",
            UniformValue::Vector2(lights[0].world_pos.coords.into()),
        )?;

        unsafe {
            self.shader.draw(
                &batch.buffers().vertices,
                &batch.buffers().elements,
                0..batch.num_elements(),
                batch.geometry_mode(),
            )?;
        }

        // FIXME: Unbind shadow map

        Ok(())
    }
}
