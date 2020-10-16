//! 2D shadow mapping for multiple light sources.
//!
//! The implementation follows https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php
//! with some modifications.

use golem::{
    blend::{BlendEquation, BlendFactor, BlendFunction, BlendMode, BlendOperation},
    Attribute, AttributeType, ColorFormat, Dimension, GeometryMode, NumberType, ShaderDescription,
    ShaderProgram, Surface, Texture, TextureFilter, TextureWrap, Uniform, UniformType,
    UniformValue,
};

use crate::{
    draw::{AsBuffersSlice, Batch, BuffersSlice, ColorVertex, Quad, Vertex},
    geom::matrix3_to_flat_array,
    Color, Context, Error, Matrix3, Point2, Point3, Vector2, Vector3,
};

pub struct LineSegment {
    pub world_pos_p: Point2,
    pub world_pos_q: Point2,
    pub order: f32,
    pub ignore_light_offset: f32,
}

impl Vertex for LineSegment {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos_p", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_world_pos_q", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_order", AttributeType::Scalar),
            Attribute::new("a_ignore_light_offset", AttributeType::Scalar),
        ]
    }

    fn num_values() -> usize {
        2 * 2 + 1 + 1
    }

    fn append(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos_p.x,
            self.world_pos_p.y,
            self.world_pos_q.x,
            self.world_pos_q.y,
            self.order,
            self.ignore_light_offset,
        ])
    }
}

pub struct LightAreaVertex {
    pub world_pos: Point2,
    pub light: Light,
    pub light_offset: f32,
}

impl Vertex for LightAreaVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_light_world_pos", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_light_params", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_light_color", AttributeType::Vector(Dimension::D4)),
            Attribute::new("a_light_offset", AttributeType::Scalar),
        ]
    }

    fn num_values() -> usize {
        2 + 2 + 3 + 4 + 1
    }

    fn append(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.light.world_pos.x,
            self.light.world_pos.y,
            self.light.radius,
            self.light.angle,
            self.light.angle_size,
            self.light.color.x,
            self.light.color.y,
            self.light.color.z,
            self.light.color.w,
            self.light_offset,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Light {
    pub world_pos: Point2,
    pub radius: f32,
    pub angle: f32,
    pub angle_size: f32,
    pub color: Color,
}

impl Light {
    pub fn params(&self) -> Vector3 {
        Vector3::new(self.radius, self.angle, self.angle_size)
    }

    /// Returns a quad that contains the maximal area that the light can reach.
    pub fn quad(&self) -> Quad {
        // TODO: Use angles to return quads that are tighter fits.
        Quad::axis_aligned(
            Point3::new(self.world_pos.x, self.world_pos.y, 0.0),
            2.0 * self.radius * Vector2::new(1.0, 1.0),
        )
    }
}

pub struct ShadowMap {
    resolution: usize,
    max_num_lights: usize,

    shadow_map: Surface,
    shadow_map_shader: ShaderProgram,

    light_surface: Surface,
    light_surface_shader: ShaderProgram,
    light_area_batch: Batch<LightAreaVertex>,
}

impl ShadowMap {
    fn new_shadow_map(
        ctx: &Context,
        resolution: usize,
        max_num_lights: usize,
    ) -> Result<Surface, Error> {
        let mut shadow_map_texture = Texture::new(ctx.golem_ctx())?;
        shadow_map_texture.set_image(
            None,
            resolution as u32,
            max_num_lights as u32,
            ColorFormat::RGBA,
        );
        shadow_map_texture.set_magnification(TextureFilter::Nearest)?;
        shadow_map_texture.set_minification(TextureFilter::Nearest)?;
        shadow_map_texture.set_wrap_h(TextureWrap::ClampToEdge)?;
        shadow_map_texture.set_wrap_v(TextureWrap::ClampToEdge)?;

        Ok(Surface::new(ctx.golem_ctx(), shadow_map_texture)?)
    }

    fn new_light_surface(ctx: &Context) -> Result<Surface, Error> {
        log::info!(
            "Creating new light surface for screen {:?}",
            ctx.draw().screen()
        );

        let mut light_texture = Texture::new(ctx.golem_ctx())?;
        // TODO: Make screen resolution u32
        light_texture.set_image(
            None,
            ctx.draw().screen().size.x as u32,
            ctx.draw().screen().size.y as u32,
            ColorFormat::RGBA,
        );
        light_texture.set_magnification(TextureFilter::Nearest)?;
        light_texture.set_minification(TextureFilter::Nearest)?;
        light_texture.set_wrap_h(TextureWrap::ClampToEdge)?;
        light_texture.set_wrap_v(TextureWrap::ClampToEdge)?;

        Ok(Surface::new(ctx.golem_ctx(), light_texture)?)
    }

    pub fn new(ctx: &Context, resolution: usize, max_num_lights: usize) -> Result<Self, Error> {
        let shadow_map = Self::new_shadow_map(ctx, resolution, max_num_lights)?;
        let light_surface = Self::new_light_surface(ctx)?;

        let shadow_map_shader = ShaderProgram::new(
            ctx.golem_ctx(),
            ShaderDescription {
                vertex_input: &LineSegment::attributes(),
                fragment_input: &[
                    Attribute::new("v_edge", AttributeType::Vector(Dimension::D4)),
                    Attribute::new("v_angle", AttributeType::Scalar),
                ],
                uniforms: &[
                    Uniform::new(
                        "light_world_pos",
                        UniformType::Vector(NumberType::Float, Dimension::D2),
                    ),
                    Uniform::new("light_radius", UniformType::Scalar(NumberType::Float)),
                    Uniform::new("light_offset", UniformType::Scalar(NumberType::Float)),
                ],
                vertex_shader: r#"
                float angle_to_light(vec2 world_pos) {
                    vec2 delta = world_pos - light_world_pos;
                    return atan(delta.y, delta.x);
                }

                const float PI = 3.141592;

                void main() {
                    if (light_offset == a_ignore_light_offset) {
                        gl_Position = vec4(-10.0, -10.0, -10.0, 1.0);
                        return;
                    }

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
                        light_offset * 2.0 - 1.0,
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
                        light_world_pos + vec2(cos(v_angle) * light_radius, sin(v_angle) * light_radius),
                        v_edge.xy,
                        v_edge.zw
                    );

                    gl_FragColor = vec4(t, t, t, t);
                }
                "#,
            },
        )?;

        let light_surface_shader = ShaderProgram::new(
            ctx.golem_ctx(),
            ShaderDescription {
                vertex_input: &LightAreaVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_delta", AttributeType::Vector(Dimension::D2)),
                    Attribute::new("v_light_params", AttributeType::Vector(Dimension::D3)),
                    Attribute::new("v_light_color", AttributeType::Vector(Dimension::D4)),
                    Attribute::new("v_light_offset", AttributeType::Scalar),
                ],
                uniforms: &[
                    Uniform::new("mat_projection_view", UniformType::Matrix(Dimension::D3)),
                    Uniform::new("shadow_map", UniformType::Sampler2D),
                    Uniform::new(
                        "shadow_map_resolution",
                        UniformType::Scalar(NumberType::Float),
                    ),
                ],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos, 1.0);
                    gl_Position = vec4(p.xy, 0.0, 1.0);
                    v_delta = a_world_pos.xy - a_light_world_pos;
                    v_light_params = a_light_params;
                    v_light_color = a_light_color;
                    v_light_offset = a_light_offset;
                }
                "#,
                fragment_shader: r#"
                void main() {
                    float angle = atan(v_delta.y, v_delta.x);
                    float dist_to_light = length(v_delta);

                    float light_radius = v_light_params.x;

                    vec2 tex_coords = vec2(angle / (2.0 * 3.141592) + 0.5, v_light_offset);
                    vec2 texel = vec2(1.0 / shadow_map_resolution, 0.0);

                    float dist1 = texture(shadow_map, tex_coords).r * light_radius;
                    float dist2 = texture(shadow_map, tex_coords - 2.0 * texel).r * light_radius;
                    float dist3 = texture(shadow_map, tex_coords + 2.0 * texel).r * light_radius;

                    float visibility = step(dist_to_light, dist1) * 0.5
                        + step(dist_to_light, dist2) * 0.25
                        + step(dist_to_light, dist3) * 0.25;

                    visibility *= pow(1.0 - dist_to_light / light_radius, 2.0);

                    float angle_diff = mod(abs(angle - v_light_params.y), 2.0 * 3.141592);
                    if (angle_diff > 3.141592)
                        angle_diff = 2.0 * 3.141592 - angle_diff;

                    visibility *= pow(1.0 - clamp(angle_diff / v_light_params.z, 0.0, 1.0), 0.5); //step(angle_diff, v_light_params.z);

                    vec3 color = v_light_color.rgb * visibility;

                    gl_FragColor = vec4(color, v_light_color.a);
                }
                "#,
            },
        )?;

        let light_area_batch = Batch::new(ctx, GeometryMode::Triangles)?;

        Ok(Self {
            resolution,
            max_num_lights,
            shadow_map,
            shadow_map_shader,
            light_surface,
            light_surface_shader,
            light_area_batch,
        })
    }

    fn fill_light_area_batch(&mut self, lights: &[Light]) {
        self.light_area_batch.clear();

        for (light_idx, light) in lights.iter().enumerate() {
            let quad = light.quad();

            for corner in &quad.corners {
                self.light_area_batch.push_vertex(&LightAreaVertex {
                    world_pos: Point2::new(corner.x, corner.y),
                    light: light.clone(),
                    light_offset: self.light_offset(light_idx),
                });
            }

            for offset in Quad::TRIANGLE_INDICES {
                self.light_area_batch
                    .push_element(light_idx as u32 * 4 + offset);
            }
        }

        self.light_area_batch.flush();
    }

    pub fn build<'a>(
        &'a mut self,
        ctx: &'a Context,
        projection: &Matrix3,
        view: &Matrix3,
        lights: &'a [Light],
    ) -> Result<BuildShadowMap<'a>, Error> {
        if ctx.draw().screen().size.x as u32 != self.light_surface.width().unwrap()
            || ctx.draw().screen().size.y as u32 != self.light_surface.height().unwrap()
        {
            // Screen surface has been resized, so we also need to recreate
            // the light surface.
            self.light_surface = Self::new_light_surface(ctx)?;
        }

        // Already upload the light area data to the GPU. This will be used for
        // rendering the lights in screen space.
        self.fill_light_area_batch(lights);

        // Clear the shadow map to maximal distance, i.e. 1.
        self.shadow_map.bind();
        ctx.golem_ctx()
            .set_viewport(0, 0, self.resolution as u32, self.max_num_lights as u32);
        ctx.golem_ctx().set_clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.golem_ctx().clear();

        Ok(BuildShadowMap {
            this: self,
            ctx,
            lights,
            projection: *projection,
            view: *view,
        })
    }

    pub fn light_offset(&self, index: usize) -> f32 {
        (index as f32 + 0.5) / self.max_num_lights as f32
    }
}

#[must_use]
pub struct BuildShadowMap<'a> {
    this: &'a mut ShadowMap,
    ctx: &'a Context,
    lights: &'a [Light],
    projection: Matrix3,
    view: Matrix3,
}

impl<'a> BuildShadowMap<'a> {
    pub fn draw_occluder_batch(self, batch: &mut Batch<LineSegment>) -> Result<Self, Error> {
        assert!(batch.geometry_mode() == GeometryMode::Lines);
        assert!(
            self.lights.len() <= self.this.max_num_lights,
            "Too many lights in ShadowMap::draw_occluder_batch: Got {} vs. max_num_lights {}",
            self.lights.len(),
            self.this.max_num_lights,
        );

        batch.flush();

        self.ctx.golem_ctx().set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Min),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

        // TODO: We should be able to batch the light draw calls

        for (light_idx, light) in self.lights.iter().enumerate() {
            self.this.shadow_map_shader.bind();
            self.this.shadow_map_shader.set_uniform(
                "light_world_pos",
                UniformValue::Vector2(light.world_pos.coords.into()),
            )?;
            self.this
                .shadow_map_shader
                .set_uniform("light_radius", UniformValue::Float(light.radius))?;
            self.this.shadow_map_shader.set_uniform(
                "light_offset",
                UniformValue::Float(self.this.light_offset(light_idx)),
            )?;

            unsafe {
                self.this.shadow_map_shader.draw(
                    &batch.buffers().vertices,
                    &batch.buffers().elements,
                    0..batch.buffers().num_elements,
                    GeometryMode::Lines,
                )?;
            }
        }

        self.ctx.golem_ctx().set_blend_mode(None);

        Ok(self)
    }

    pub fn finish(self) -> Result<(), Error> {
        let golem_ctx = self.ctx.golem_ctx();

        //Surface::unbind(self.ctx.golem_ctx());
        self.this.light_surface.bind();

        golem_ctx.set_viewport(
            0,
            0,
            self.this.light_surface.width().unwrap(),
            self.this.light_surface.height().unwrap(),
        );
        golem_ctx.set_clear_color(0.0, 0.0, 0.0, 1.0);
        golem_ctx.clear();

        self.ctx.golem_ctx().set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Add),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

        let projection_view = self.projection * self.view;

        unsafe {
            self.this
                .shadow_map
                .borrow_texture()
                .unwrap()
                .set_active(std::num::NonZeroU32::new(1).unwrap());
        }

        self.this.light_surface_shader.bind();
        self.this.light_surface_shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;
        self.this
            .light_surface_shader
            .set_uniform("shadow_map", UniformValue::Int(1))?;
        self.this.light_surface_shader.set_uniform(
            "shadow_map_resolution",
            UniformValue::Float(self.this.resolution as f32),
        )?;

        unsafe {
            self.this.light_surface_shader.draw(
                &self.this.light_area_batch.buffers().vertices,
                &self.this.light_area_batch.buffers().elements,
                0..self.this.light_area_batch.buffers().num_elements,
                GeometryMode::Triangles,
            )?;
        }

        self.ctx.golem_ctx().set_blend_mode(None);

        Surface::unbind(self.ctx.golem_ctx());

        Ok(())
    }
}

pub struct ShadowedColorPass {
    shader: ShaderProgram,
}

impl ShadowedColorPass {
    pub fn new(ctx: &Context) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            ctx.golem_ctx(),
            ShaderDescription {
                vertex_input: &ColorVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_tex_coords", AttributeType::Vector(Dimension::D2)),
                    Attribute::new("v_color", AttributeType::Vector(Dimension::D4)),
                ],
                uniforms: &[
                    Uniform::new("mat_projection_view", UniformType::Matrix(Dimension::D3)),
                    Uniform::new("light_surface", UniformType::Sampler2D),
                    Uniform::new(
                        "ambient_light",
                        UniformType::Vector(NumberType::Float, Dimension::D4),
                    ),
                ],
                vertex_shader: r#"
                void main() {
                    vec3 p = mat_projection_view * vec3(a_world_pos.xy, 1.0);
                    gl_Position = vec4(p.xy, a_world_pos.z, 1.0);
                    v_color = a_color;
                    v_tex_coords = (gl_Position.xy + vec2(1.0, 1.0)) / 2.0;
                }
                "#,
                fragment_shader: r#"
                void main() {
                    vec3 light = texture(light_surface, v_tex_coords).rgb;
                    vec3 reflect = ambient_light.rgb + light * v_color.rgb;
                    gl_FragColor = vec4(
                        pow(reflect, vec3(1.0/2.2)),
                        v_color.a
                    );
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
        ambient_light: Color,
        shadow_map: &ShadowMap,
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
                ambient_light,
                shadow_map,
                batch.buffers().as_buffers_slice(),
                batch.geometry_mode(),
            )
        }
    }

    pub unsafe fn draw_buffers(
        &mut self,
        projection: &Matrix3,
        view: &Matrix3,
        ambient_light: Color,
        shadow_map: &ShadowMap,
        buffers: BuffersSlice<ColorVertex>,
        geometry_mode: GeometryMode,
    ) -> Result<(), Error> {
        let projection_view = projection * view;

        shadow_map
            .light_surface
            .borrow_texture()
            .unwrap()
            .set_active(std::num::NonZeroU32::new(1).unwrap());

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_flat_array(&projection_view)),
        )?;
        self.shader.set_uniform(
            "ambient_light",
            UniformValue::Vector4(ambient_light.coords.into()),
        )?;
        self.shader
            .set_uniform("light_surface", UniformValue::Int(1))?;

        self.shader.draw(
            buffers.vertices,
            buffers.elements,
            0..buffers.num_elements,
            geometry_mode,
        )?;

        // FIXME: Unbind light surface

        Ok(())
    }
}
