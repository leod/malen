use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Glsl, Texture, Uniform},
    glsl, program, Color4,
};

use super::{
    super::{GlobalLightProps, LightPipelineParams},
    GLOBAL_LIGHT_PROPS_BLOCK_BINDING,
};

program! {
    program ComposeWithIndirectProgram
    params {
        params: LightPipelineParams,
    }
    uniforms {
        global_light_props: GlobalLightProps = GLOBAL_LIGHT_PROPS_BLOCK_BINDING,
    }
    samplers {
        screen_albedo: Sampler2,
        screen_normals: Sampler2,
        screen_occlusion: Sampler2,
        screen_reflector: Sampler2,
        screen_light: Sampler2,
    }
    attributes {
        a: SpriteVertex,
    }
    defines [
        pi => std::f32::consts::PI,
        num_tracing_cones => params.num_tracing_cones,
        num_tracing_steps => params.num_tracing_steps,
    ]
    includes [CONE_TRACING_SOURCE]
    vertex glsl! {
        out vec2 v_tex_coords;

        void main() {
            gl_Position = vec4(a_position.xyz, 1.0);
            v_tex_coords = a_tex_coords;
        }
    }
    fragment glsl! {
        in vec2 v_tex_coords;
        out vec4 f_color;

        void main() {
            vec3 direct_light = texture(screen_light, v_tex_coords).rgb;
            vec3 indirect_light = calc_indirect_diffuse_light(v_tex_coords);
            vec3 light = direct_light + indirect_light;

            vec4 albedo = texture(screen_albedo, v_tex_coords);
            vec3 diffuse = vec3(albedo) * (light + global_light_props.ambient);

            vec3 mapped = diffuse / (diffuse + vec3(1.0));
            f_color = vec4(pow(mapped, vec3(1.0 / global_light_props.gamma)), 1.0);
        }
    }
}

const CONE_TRACING_SOURCE: Glsl = glsl! {
    vec3 trace_cone(vec2 origin, vec2 dir) {
        const float cone_angle = 2.0 * {pi} / {num_tracing_cones}.0;
        const float diameter_scale = 2.0 * tan(cone_angle / 2.0);

        float t = global_light_props.indirect_initial_offset;
        float occlusion = 0.0;
        vec3 color = vec3(0.0, 0.0, 0.0);
        vec2 screen_size = vec2(textureSize(screen_occlusion, 0));

        for (int i = 0; i < {num_tracing_steps} && occlusion <= 0.9; i += 1) {
            float cone_diameter = diameter_scale * t;
            vec2 p = origin + dir / screen_size * t;
            if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0)
                break;

            float mip_level = clamp(log2(cone_diameter), 0.0, 10.0);
            float sample_occlusion = textureLod(screen_occlusion, p, mip_level).r;
            vec3 sample_color = textureLod(screen_reflector, p, mip_level).rgb;

            if (sample_occlusion > 0.0) {
                sample_color *= global_light_props.indirect_intensity;

                color += (1.0 - occlusion) * sample_color;
                occlusion += (1.0 - occlusion) * sample_occlusion;
            }

            t += global_light_props.indirect_step_factor * cone_diameter;
        }

        return color;
    }

    vec3 calc_indirect_diffuse_light(vec2 origin) {
        const int n = {num_tracing_cones};
        const float dangle = 2.0 * {pi} / float(n);

        float self_occlusion = textureLod(screen_occlusion, origin, 0.0).r;
        float self_occlusion_scale =
            1.0 - global_light_props.indirect_self_occlusion * self_occlusion;

        vec3 normal_value = texture(screen_normals, origin).xyz;
        vec3 normal = normal_value * 2.0 - 1.0;
        normal.y = -normal.y;
        normal = normalize(normal);

        vec3 color = vec3(0.0, 0.0, 0.0);
        float angle = 0.0;

        for (int i = 0; i < n; i += 1) {
            vec2 dir = vec2(cos(angle), sin(angle));
            float scale = normal_value == vec3(0.0) ?
                1.0 :
                max(dot(normalize(vec3(-dir, global_light_props.indirect_z)), normal), 0.0);

            color += scale * trace_cone(origin, dir);
            angle += dangle;
        }

        return self_occlusion_scale * color / float(n);
    }
};

pub struct ComposeWithIndirectPass {
    screen_rect: Mesh<SpriteVertex>,
    program: ComposeWithIndirectProgram,
}

impl ComposeWithIndirectPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let screen_rect = Mesh::from_geometry(
            gl.clone(),
            Sprite {
                rect: Rect {
                    center: Point2::origin(),
                    size: Vector2::new(2.0, 2.0),
                },
                depth: 0.0,
                tex_rect: Rect::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0)),
                color: Color4::new(1.0, 1.0, 1.0, 1.0),
            },
        )?;

        let program = ComposeWithIndirectProgram::new(gl, params)?;

        Ok(Self {
            screen_rect,
            program,
        })
    }

    pub fn draw(
        &self,
        global_light_props: &Uniform<GlobalLightProps>,
        screen_albedo: &Texture,
        screen_normal: &Texture,
        screen_occlusion: &Texture,
        screen_reflector: &Texture,
        screen_light: &Texture,
    ) {
        gl::draw(
            &self.program,
            global_light_props,
            [
                screen_albedo,
                screen_normal,
                screen_occlusion,
                screen_reflector,
                screen_light,
            ],
            self.screen_rect.draw_unit(),
            &DrawParams::default(),
        );
    }
}
