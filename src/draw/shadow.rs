//! 2D shadow mapping for multiple light sources.
//!
//! The implementation follows https://www.gamasutra.com/blogs/RobWare/20180226/313491/Fast_2D_shadows_in_Unity_using_1D_shadow_mapping.php
//! with some modifications.

use golem::{
    blend::{BlendEquation, BlendFactor, BlendFunction, BlendMode, BlendOperation},
    Attribute, AttributeType, ColorFormat, Dimension, GeometryMode, GolemError, NumberType,
    ShaderDescription, ShaderProgram, Surface, Texture, TextureFilter, TextureWrap, Uniform,
    UniformType, UniformValue,
};
use nalgebra::{Matrix3, Point2, Vector2, Vector3};

use crate::{
    draw::{Batch, ColVertex, DrawUnit, Geometry, Quad, TriBatch, Vertex},
    math::matrix3_to_array,
    Canvas, Color3, Error, Screen,
};

pub struct LineSegment {
    pub world_pos_p: Point2<f32>,
    pub world_pos_q: Point2<f32>,
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

    fn write(&self, out: &mut Vec<f32>) {
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

impl Geometry for LineSegment {
    type Vertex = LineSegment;

    fn mode() -> GeometryMode {
        GeometryMode::Lines
    }
}

pub type OccluderBatch = Batch<LineSegment>;

impl Batch<LineSegment> {
    pub fn push_occluder_line(
        &mut self,
        line_p: Point2<f32>,
        line_q: Point2<f32>,
        ignore_light_offset: Option<f32>,
    ) {
        let first_idx = self.next_index() as u32;

        let ignore_light_offset = ignore_light_offset.unwrap_or(-1.0);

        self.push_vertex(&LineSegment {
            world_pos_p: line_p,
            world_pos_q: line_q,
            order: 0.0,
            ignore_light_offset,
        });
        self.push_vertex(&LineSegment {
            world_pos_p: line_q,
            world_pos_q: line_p,
            order: 1.0,
            ignore_light_offset,
        });

        self.push_vertex(&LineSegment {
            world_pos_p: line_p,
            world_pos_q: line_q,
            order: 2.0,
            ignore_light_offset,
        });
        self.push_vertex(&LineSegment {
            world_pos_p: line_q,
            world_pos_q: line_p,
            order: 3.0,
            ignore_light_offset,
        });

        self.push_element(first_idx + 0);
        self.push_element(first_idx + 1);
        self.push_element(first_idx + 2);
        self.push_element(first_idx + 3);
    }

    pub fn push_occluder_quad(&mut self, quad: &Quad, ignore_light_offset: Option<f32>) {
        self.push_occluder_line(quad.corners[0], quad.corners[1], ignore_light_offset);
        self.push_occluder_line(quad.corners[1], quad.corners[2], ignore_light_offset);
        self.push_occluder_line(quad.corners[2], quad.corners[3], ignore_light_offset);
        self.push_occluder_line(quad.corners[3], quad.corners[0], ignore_light_offset);
    }
}

pub struct LightAreaVertex {
    pub world_pos: Point2<f32>,
    pub light: Light,
    pub light_offset: f32,
}

impl Vertex for LightAreaVertex {
    fn attributes() -> Vec<Attribute> {
        vec![
            Attribute::new("a_world_pos", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_light_world_pos", AttributeType::Vector(Dimension::D2)),
            Attribute::new("a_light_params", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_light_color", AttributeType::Vector(Dimension::D3)),
            Attribute::new("a_light_offset", AttributeType::Scalar),
        ]
    }

    fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[
            self.world_pos.x,
            self.world_pos.y,
            self.light.world_pos.x,
            self.light.world_pos.y,
            self.light.radius,
            self.light.angle,
            self.light.angle_size,
            self.light.color.r,
            self.light.color.g,
            self.light.color.b,
            self.light_offset,
        ])
    }
}

#[derive(Debug, Clone)]
pub struct Light {
    pub world_pos: Point2<f32>,
    pub radius: f32,
    pub angle: f32,
    pub angle_size: f32,
    pub color: Color3,
}

impl Light {
    pub fn params(&self) -> Vector3<f32> {
        Vector3::new(self.radius, self.angle, self.angle_size)
    }

    /// Returns a quad that contains the maximal area that the light can reach.
    pub fn quad(&self) -> Quad {
        // TODO: Use angles to return quads that are tighter fits.
        Quad::axis_aligned(self.world_pos, 2.0 * self.radius * Vector2::new(1.0, 1.0))
    }
}

fn light_offset(max_num_lights: usize, index: usize) -> f32 {
    (index as f32 + 0.5) / max_num_lights as f32
}

pub struct ShadowMap {
    resolution: usize,
    max_num_lights: usize,

    shadow_map: Surface,
    shadow_map_shader: ShaderProgram,

    light_surface: Surface,
    light_surface_shader: ShaderProgram,
    light_area_batch: TriBatch<LightAreaVertex>,
}

impl ShadowMap {
    fn new_shadow_map(
        canvas: &Canvas,
        resolution: usize,
        max_num_lights: usize,
    ) -> Result<Surface, Error> {
        let mut shadow_map_texture = Texture::new(canvas.golem_ctx())?;
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

        Ok(Surface::new(canvas.golem_ctx(), shadow_map_texture)?)
    }

    fn light_surface_size(canvas: &Canvas) -> Vector2<u32> {
        Vector2::new(
            canvas
                .screen()
                .physical_size
                .x
                .min(canvas.caps().max_texture_size),
            canvas
                .screen()
                .physical_size
                .y
                .min(canvas.caps().max_texture_size),
        )
    }

    fn new_light_surface(canvas: &Canvas) -> Result<Surface, Error> {
        let size = Self::light_surface_size(canvas);

        log::info!(
            "Creating new light surface for screen {:?} with size {:?}",
            canvas.screen(),
            size,
        );
        log::info!(
            "Maximum allowed texture size: {}",
            canvas.caps().max_texture_size
        );

        let mut light_texture = Texture::new(canvas.golem_ctx())?;
        light_texture.set_image(None, size.x, size.y, ColorFormat::RGBA);
        light_texture.set_magnification(TextureFilter::Nearest)?;
        light_texture.set_minification(TextureFilter::Nearest)?;
        light_texture.set_wrap_h(TextureWrap::ClampToEdge)?;
        light_texture.set_wrap_v(TextureWrap::ClampToEdge)?;

        Ok(Surface::new(canvas.golem_ctx(), light_texture)?)
    }

    pub fn new(canvas: &Canvas, resolution: usize, max_num_lights: usize) -> Result<Self, Error> {
        let shadow_map = Self::new_shadow_map(canvas, resolution, max_num_lights)?;
        let light_surface = Self::new_light_surface(canvas)?;

        let shadow_map_shader = ShaderProgram::new(
            canvas.golem_ctx(),
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
                float ray_line_segment_intersection(
                    vec2 o,
                    vec2 d,
                    vec2 p,
                    vec2 q
                ) {
                    /**

                    ray(s) = o + d * s             (0 <= s)
                    line(t) = p + (q - p) * t      (0 <= t <= 1)

                    ray(s) = line(t)

                    <=> o + d * s = p + (q - p) * t

                    <=> d * s + (p - q) * t = p - o

                    <=> M * [[s], [t]] = p - o
                    where M = [[d.x, d.y], [p.x - q.x, p.y - q.y]] 

                    <=> [[s], [t]] = M^-1 (p - o)   (if M is invertible)
                    where M^-1 = 1.0 / det(M) * [[p.y - q.y, -d.y], [q.x - p.x, d.x]]

                    **/

                    float det = d.x * (p.y - q.y) + d.y * (q.x - p.x);

                    if (abs(det) < 0.0000001)
                        return 1.0;

                    mat2 m_inv = mat2(
                        p.y - q.y, 
                        -d.y,
                        q.x - p.x, 
                        d.x
                    );

                    vec2 time = 1.0 / det * m_inv * (p - o);

                    float s = time.x;
                    float t = time.y;

                    if (s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0) {
                        return s;
                    } else {
                        return 1.0;
                    }
                }

                void main() {
                    float t = ray_line_segment_intersection(
                        light_world_pos,
                        vec2(cos(v_angle), sin(v_angle)) * light_radius,
                        v_edge.xy,
                        v_edge.zw
                    );

                    gl_FragColor = vec4(t, t, t, t);
                }
                "#,
            },
        )?;

        let light_surface_shader = ShaderProgram::new(
            canvas.golem_ctx(),
            ShaderDescription {
                vertex_input: &LightAreaVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_delta", AttributeType::Vector(Dimension::D2)),
                    Attribute::new("v_light_params", AttributeType::Vector(Dimension::D3)),
                    Attribute::new("v_light_color", AttributeType::Vector(Dimension::D3)),
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

                    float dist1 = texture(shadow_map, tex_coords).r * light_radius + 1.0;
                    float dist2 = texture(shadow_map, tex_coords - 1.0 * texel).r * light_radius + 1.0;
                    float dist3 = texture(shadow_map, tex_coords + 1.0 * texel).r * light_radius + 1.0;

                    float visibility = step(dist_to_light, dist1);
                    visibility *= 0.5;
                    visibility += step(dist_to_light, dist2) * 0.25;
                    visibility += step(dist_to_light, dist3) * 0.25;

                    visibility *= pow(1.0 - dist_to_light / light_radius, 2.0);

                    float angle_diff = mod(abs(angle - v_light_params.y), 2.0 * 3.141592);
                    if (angle_diff > 3.141592)
                        angle_diff = 2.0 * 3.141592 - angle_diff;

                    //visibility *= pow(1.0 - clamp(angle_diff / v_light_params.z, 0.0, 1.0), 0.5); 
                    visibility *= step(angle_diff, v_light_params.z);

                    vec3 color = v_light_color * visibility;

                    gl_FragColor = vec4(color, 1.0);
                }
                "#,
            },
        )?;

        let light_area_batch = TriBatch::new(canvas)?;

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
                    light_offset: light_offset(self.max_num_lights, light_idx),
                });
            }

            for offset in Quad::TRIANGLE_INDICES {
                self.light_area_batch
                    .push_element(light_idx as u32 * 4 + offset);
            }
        }
    }

    pub fn build<'a>(
        &'a mut self,
        canvas: &'a Canvas,
        view: &'a Matrix3<f32>,
        lights: &'a [Light],
    ) -> Result<BuildShadowMap<'a>, Error> {
        let light_surface_size = Self::light_surface_size(canvas);
        if light_surface_size.x != self.light_surface.width().unwrap()
            || light_surface_size.y != self.light_surface.height().unwrap()
        {
            // Screen surface has been resized, so we also need to recreate
            // the light surface.
            self.light_surface = Self::new_light_surface(canvas)?;
        }

        // Already upload the light area data to the GPU. This will be used for
        // rendering the lights in screen space.
        self.fill_light_area_batch(lights);

        // Clear the shadow map to maximal distance, i.e. 1.
        self.shadow_map.bind();
        canvas.set_viewport(
            Point2::origin(),
            Vector2::new(self.resolution as u32, self.max_num_lights as u32),
        );
        canvas.golem_ctx().set_clear_color(1.0, 1.0, 1.0, 1.0);
        canvas.golem_ctx().clear();

        Ok(BuildShadowMap {
            this: self,
            canvas,
            lights,
            view: view.clone(),
        })
    }

    pub fn light_offset(&self, index: usize) -> f32 {
        (index as f32 + 0.5) / self.max_num_lights as f32
    }

    pub fn shadow_map(&self) -> &Surface {
        &self.shadow_map
    }

    pub fn light_surface(&self) -> &Surface {
        &self.light_surface
    }
}

#[must_use]
pub struct BuildShadowMap<'a> {
    this: &'a mut ShadowMap,
    canvas: &'a Canvas,
    lights: &'a [Light],
    view: Matrix3<f32>,
}

impl<'a> BuildShadowMap<'a> {
    pub fn draw_occluders(self, draw_unit: &DrawUnit<LineSegment>) -> Result<Self, Error> {
        assert!(draw_unit.geometry_mode() == GeometryMode::Lines);
        assert!(
            self.lights.len() <= self.this.max_num_lights,
            "Too many lights in ShadowMap::draw_occluder_batch: Got {} vs. max_num_lights {}",
            self.lights.len(),
            self.this.max_num_lights,
        );

        self.canvas.golem_ctx().set_blend_mode(Some(BlendMode {
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

            draw_unit.draw(&self.this.shadow_map_shader)?;
        }

        self.canvas.golem_ctx().set_blend_mode(None);

        Ok(self)
    }

    pub fn finish(self) -> Result<(), Error> {
        let screen = self.canvas.screen();
        let golem_ctx = self.canvas.golem_ctx();

        //Surface::unbind(self.ctx.golem_ctx());
        self.this.light_surface.bind();

        golem_ctx.set_clear_color(0.0, 0.0, 0.0, 1.0);
        golem_ctx.clear();

        let clipped_screen = Screen {
            physical_size: Vector2::new(
                self.this.light_surface.width().unwrap(),
                self.this.light_surface.height().unwrap(),
            ),
            ..screen
        };
        let transform = clipped_screen.orthographic_projection() * self.view;
        self.canvas
            .set_viewport(Point2::origin(), clipped_screen.physical_size);

        golem_ctx.set_blend_mode(Some(BlendMode {
            equation: BlendEquation::Same(BlendOperation::Add),
            function: BlendFunction::Same {
                source: BlendFactor::One,
                destination: BlendFactor::One,
            },
            ..Default::default()
        }));

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
            UniformValue::Matrix3(matrix3_to_array(&transform)),
        )?;
        self.this
            .light_surface_shader
            .set_uniform("shadow_map", UniformValue::Int(1))?;
        if let Err(GolemError::NoSuchUniform(_)) = self.this.light_surface_shader.set_uniform(
            "shadow_map_resolution",
            UniformValue::Float(self.this.resolution as f32),
        ) {
            // Ignore missing shadow_map_resolution error, if PCF is disabled.
        }

        self.this
            .light_area_batch
            .draw(&self.this.light_surface_shader)?;

        // Clean up
        self.canvas.golem_ctx().set_blend_mode(None);
        Surface::unbind(self.canvas.golem_ctx());
        self.canvas
            .set_viewport(Point2::origin(), self.canvas.screen().physical_size);

        Ok(())
    }
}

pub struct ShadowColPass {
    shader: ShaderProgram,
}

impl ShadowColPass {
    pub fn new(canvas: &Canvas) -> Result<Self, Error> {
        let shader = ShaderProgram::new(
            canvas.golem_ctx(),
            ShaderDescription {
                vertex_input: &ColVertex::attributes(),
                fragment_input: &[
                    Attribute::new("v_tex_coords", AttributeType::Vector(Dimension::D2)),
                    Attribute::new("v_color", AttributeType::Vector(Dimension::D4)),
                ],
                uniforms: &[
                    Uniform::new("mat_projection_view", UniformType::Matrix(Dimension::D3)),
                    Uniform::new("light_surface", UniformType::Sampler2D),
                    Uniform::new(
                        "ambient_light",
                        UniformType::Vector(NumberType::Float, Dimension::D3),
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
                    vec3 reflect = (ambient_light + light) * v_color.rgb;
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

    pub fn draw(
        &mut self,
        transform: &Matrix3<f32>,
        ambient_light: Color3,
        shadow_map: &ShadowMap,
        draw_unit: &DrawUnit<ColVertex>,
    ) -> Result<(), Error> {
        unsafe {
            shadow_map
                .light_surface
                .borrow_texture()
                .unwrap()
                .set_active(std::num::NonZeroU32::new(1).unwrap());
        }

        self.shader.bind();
        self.shader.set_uniform(
            "mat_projection_view",
            UniformValue::Matrix3(matrix3_to_array(transform)),
        )?;
        self.shader
            .set_uniform("ambient_light", UniformValue::Vector3(ambient_light.into()))?;
        self.shader
            .set_uniform("light_surface", UniformValue::Int(1))?;

        draw_unit.draw(&self.shader)?;

        // FIXME: Unbind light surface

        Ok(())
    }
}
