use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteBatch, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Program, ProgramDef, Texture, Uniform},
    light::IndirectLightPipelineParams,
};

use super::{super::def::GlobalLightParamsBlock, GLOBAL_LIGHT_PARAMS_BLOCK_BINDING};

const VERTEX_SOURCE: &str = r#"
out vec2 v_tex_coords;

void main() {
    gl_Position = vec4(a_position.xyz, 1.0);
    v_tex_coords = a_tex_coords;
}
"#;

const CONE_TRACING_SOURCE: &str = r#"
const float PI = 3.141592;

vec3 trace_cone(
    vec2 origin,
    vec2 dir
) {
    const float cone_angle = 2.0 * PI / {num_tracing_cones}.0;
    const float diameter_scale = 2.0 * tan(cone_angle / 2.0);

    float t = global_light_params.indirect_start;
    float occlusion = 0.0;
    vec3 color = vec3(0.0, 0.0, 0.0);

    for (int i = 0; i < {num_tracing_steps} && occlusion <= 0.9; i++) {
        float cone_diameter = diameter_scale * t;
        vec2 p = origin + dir / global_light_params.screen_size * t;
        if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0)
            break;

        float mip_level = clamp(log2(cone_diameter), 0.0, 10.0);
        float sample_occlusion = textureLod(screen_occlusion, p, mip_level).r;
        vec3 sample_color = textureLod(screen_reflectors, p, mip_level).rgb;

        if (sample_occlusion > 0.0) {
            sample_color *= global_light_params.indirect_color_scale;

            // This equation (from the paper) leads to very pronounced borders in 2D, due to lack
            // of interior lighting.
            //color = occlusion * color + (1.0 - occlusion) * occlusion * 2.0 * sample_color;

            color += (1.0 - occlusion) * sample_color;
            occlusion += (1.0 - occlusion) * sample_occlusion;
        }

        t += global_light_params.indirect_step_factor * cone_diameter;
    }

    return color;
}

vec3 calc_indirect_diffuse_lighting(
    vec2 origin
) {
    const int n = {num_tracing_cones};
    const float dangle = 2.0 * PI / float(n);

    vec3 normal_value = texture(screen_normals, origin).xyz;
    vec3 normal = normal_value * 2.0 - 1.0;
    normal.y = -normal.y;

    vec3 color = vec3(0.0, 0.0, 0.0);
    float angle = 0.0;

    for (int i = 0; i < n; i++) {
        vec2 dir = vec2(cos(angle), sin(angle));
        float scale = normal_value == vec3(0.0) ?
            1.0 :
            max(dot(normalize(vec3(-dir, global_light_params.indirect_z)), normalize(normal)), 0.0);

        color += scale * trace_cone(origin, dir);
        angle += dangle;
    }

    return color / float(n);
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    vec4 albedo = texture(screen_albedo, v_tex_coords);

    vec3 direct_light = texture(screen_light, v_tex_coords).rgb;
    vec3 indirect_light = calc_indirect_diffuse_lighting(v_tex_coords);
    vec3 light = direct_light + indirect_light;

    vec3 diffuse = vec3(albedo) * (light + albedo.a * global_light_params.ambient);

    vec3 mapped = diffuse / (diffuse + vec3(1.0));

    f_color = vec4(pow(mapped, vec3(1.0 / global_light_params.gamma)), 1.0);
}
"#;

pub struct ComposeWithIndirectPass {
    screen_rect: Mesh<SpriteVertex>,
    program: Program<GlobalLightParamsBlock, SpriteVertex, 5>,
}

impl ComposeWithIndirectPass {
    pub fn new(
        gl: Rc<gl::Context>,
        params: IndirectLightPipelineParams,
    ) -> Result<Self, gl::Error> {
        let screen_rect = SpriteBatch::from_geometry(
            gl.clone(),
            Sprite {
                rect: Rect {
                    center: Point2::origin(),
                    size: Vector2::new(2.0, 2.0),
                },
                z: 0.0,
                tex_rect: Rect::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0)),
            },
        )?
        .into_mesh();

        let program_def = ProgramDef {
            uniform_blocks: [("global_light_params", GLOBAL_LIGHT_PARAMS_BLOCK_BINDING)],
            samplers: [
                "screen_albedo",
                "screen_normals",
                "screen_occlusion",
                "screen_light",
                "screen_reflectors",
            ],
            vertex_source: &VERTEX_SOURCE,
            fragment_source: &format!("{}\n{}", CONE_TRACING_SOURCE, FRAGMENT_SOURCE)
                .replace("{num_tracing_cones}", &params.num_tracing_cones.to_string())
                .replace("{num_tracing_steps}", &params.num_tracing_steps.to_string()),
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            screen_rect,
            program,
        })
    }

    pub fn draw(
        &self,
        global_light_params: &Uniform<GlobalLightParamsBlock>,
        screen_albedo: &Texture,
        screen_normal: &Texture,
        screen_occlusion: &Texture,
        screen_light: &Texture,
        screen_reflectors: &Texture,
    ) {
        gl::draw(
            &self.program,
            global_light_params,
            [
                screen_albedo,
                screen_normal,
                screen_occlusion,
                screen_light,
                screen_reflectors,
            ],
            self.screen_rect.draw_unit(),
            &DrawParams::default(),
        );
    }
}
