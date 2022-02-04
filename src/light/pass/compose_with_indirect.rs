use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Program, ProgramDef, Texture, Uniform},
    light::LightPipelineParams,
    Color4,
};

use super::{super::def::GlobalLightParamsBlock, GLOBAL_LIGHT_PARAMS_BLOCK_BINDING};

pub struct ComposeWithIndirectPass {
    screen_rect: Mesh<SpriteVertex>,
    program: Program<GlobalLightParamsBlock, SpriteVertex, 5>,
}

const UNIFORM_BLOCKS: [(&str, u32); 1] = [("params", GLOBAL_LIGHT_PARAMS_BLOCK_BINDING)];

const SAMPLERS: [&str; 5] = [
    "screen_albedo",
    "screen_normals",
    "screen_occlusion",
    "screen_reflector",
    "screen_light",
];

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

    float t = params.indirect_initial_offset;
    float occlusion = 0.0;
    vec3 color = vec3(0.0, 0.0, 0.0);
    vec2 screen_size = vec2(textureSize(screen_occlusion, 0));

    for (int i = 0; i < {num_tracing_steps} && occlusion <= 0.9; i++) {
        float cone_diameter = diameter_scale * t;
        vec2 p = origin + dir / screen_size * t;
        if (p.x < 0.0 || p.x > 1.0 || p.y < 0.0 || p.y > 1.0)
            break;

        float mip_level = log2(cone_diameter);
        float sample_occlusion = textureLod(screen_occlusion, p, mip_level).r;
        vec3 sample_color = textureLod(screen_reflector, p, mip_level).rgb;
        //sample_color = vec3(sample_occlusion, 0.0, 0.0);

        sample_color *= params.indirect_intensity;

        color += (1.0 - occlusion) * sample_color;
        //occlusion += (1.0 - occlusion) * sample_occlusion;

        // This equation (from the paper) leads to very pronounced borders in 2D, due to lack
        // of interior lighting. (This is probably due to a misunderstanding on my end.)
        //color = occlusion * color + (1.0 - occlusion) * occlusion * 2.0 * sample_color;

        t += params.indirect_step_factor * cone_diameter;
    }

    return color;
}

vec3 calc_indirect_diffuse_lighting(
    vec2 origin
) {
    const int n = {num_tracing_cones};
    const float dangle = 2.0 * PI / float(n);

    float self_occlusion = textureLod(screen_occlusion, origin, 0.0).r;
    self_occlusion = 0.0;

    vec3 normal_value = texture(screen_normals, origin).xyz;
    vec3 normal = normal_value * 2.0 - 1.0;
    normal.y = -normal.y;
    normal = normalize(normal);

    vec3 color = vec3(0.0, 0.0, 0.0);

    for (int i = 0; i < n; i++) {
        float angle = float(i) * dangle;
        vec2 dir = vec2(cos(angle), sin(angle));
        float scale = normal_value == vec3(0.0) ?
            1.0 :
            max(dot(normalize(vec3(-dir, params.indirect_z)), normal), 0.0);
        scale = 1.0;

        color += scale * trace_cone(origin, dir);
        break;
    }

    return (1.0 - params.indirect_self_occlusion * self_occlusion) * color / float(n);
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_tex_coords;
out vec4 f_color;

void main() {
    vec3 direct_light = texture(screen_light, v_tex_coords).rgb;
    vec3 indirect_light = calc_indirect_diffuse_lighting(v_tex_coords);
    vec3 light = direct_light + indirect_light;

    vec4 albedo = texture(screen_albedo, v_tex_coords);
    vec3 diffuse = vec3(albedo) * (light + params.ambient);

    vec3 mapped = diffuse / (diffuse + vec3(1.0));
    //f_color = textureLod(screen_reflector, v_tex_coords, 4.0) * 10.0;
    //f_color = vec4(pow(mapped, vec3(1.0 / params.gamma)), 1.0);
    f_color = vec4(indirect_light, 1.0); 
}
"#;

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

        let program_def = ProgramDef {
            uniform_blocks: UNIFORM_BLOCKS,
            samplers: SAMPLERS,
            vertex_source: VERTEX_SOURCE,
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
        params: &Uniform<GlobalLightParamsBlock>,
        screen_albedo: &Texture,
        screen_normal: &Texture,
        screen_occlusion: &Texture,
        screen_reflector: &Texture,
        screen_light: &Texture,
    ) {
        gl::draw(
            &self.program,
            params,
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
