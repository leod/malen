use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Program,
        ProgramDef, Texture, Uniform,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{
    super::{light_area::LightAreaVertex, GlobalLightParamsBlock, LightPipelineParams},
    GLOBAL_LIGHT_PARAMS_BLOCK_BINDING,
};

pub struct ScreenLightPass {
    program: Program<(MatricesBlock, GlobalLightParamsBlock), LightAreaVertex, 2>,
}

const UNIFORM_BLOCKS: [(&str, u32); 2] = [
    ("matrices", MATRICES_BLOCK_BINDING),
    ("params", GLOBAL_LIGHT_PARAMS_BLOCK_BINDING),
];

const SAMPLERS: [&str; 2] = ["shadow_map", "screen_normals"];

const VERTEX_SOURCE: &str = r#"
flat out vec4 v_light_params;
flat out vec3 v_light_color;
flat out float v_light_offset;
out vec2 v_screen_uv;
out vec3 v_delta;

void main() {
    v_light_params = a_light_params;
    v_light_color = a_light_color;
    v_light_offset = (float(a_light_index) + 0.5) / float({max_num_lights});

    vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_screen_uv = p.xy * 0.5 + 0.5;
    v_delta = vec3(a_position.xy, 0.0) - a_light_position;
}
"#;

pub(crate) const VISIBILITY_SOURCE: &str = r#"
float visibility(
    in sampler2D shadow_map,
    in float light_offset,
    in vec4 light_params,
    in vec2 delta
) {
    const float PI = 3.141592;
    const float DEPTH_TEXELS = 2.0;

    float light_radius = light_params.x;
    float light_angle = light_params.y;
    float light_angle_size = light_params.z;
    float light_start = light_params.w;

    float dist_to_light = length(delta);
    if (dist_to_light > light_radius)
        discard;

    float angle = atan(delta.y, delta.x);

    float fall_on = (1.0 + sin(PI * (3.0/2.0 +
        clamp(dist_to_light / light_start - 1.0, 0.0, 1.0)))) / 2.0;
    float front_light = fall_on * pow(1.0 - dist_to_light / light_radius, 2.0);
    float angle_diff = mod(abs(angle - light_angle), 2.0 * PI);
    if (angle_diff > PI)
        angle_diff = 2.0 * PI - angle_diff;

    float angle_to_border = angle_diff * 2.0 - light_angle_size
        + params.angle_fall_off_size;
    if (abs(light_angle_size - 2.0 * PI) > 0.001 && angle_to_border > 0.0) {
        float t = angle_to_border / params.angle_fall_off_size;
        front_light *= 2.0 / (1.0 + 1.0 * exp(10.0 * t));
    }

    vec2 tex_coords = vec2(angle / (2.0 * PI) + 0.5, light_offset);
    vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);

    vec2 depth3l = texture(shadow_map, tex_coords - 3.0 * DEPTH_TEXELS * texel).xy * light_radius;
    vec2 depth2l = texture(shadow_map, tex_coords - 2.0 * DEPTH_TEXELS * texel).xy * light_radius;
    vec2 depth1l = texture(shadow_map, tex_coords - 1.0 * DEPTH_TEXELS * texel).xy * light_radius;
    vec2 depth0  = texture(shadow_map, tex_coords                             ).xy * light_radius;
    vec2 depth1r = texture(shadow_map, tex_coords + 1.0 * DEPTH_TEXELS * texel).xy * light_radius;
    vec2 depth2r = texture(shadow_map, tex_coords + 2.0 * DEPTH_TEXELS * texel).xy * light_radius;
    vec2 depth3r = texture(shadow_map, tex_coords + 3.0 * DEPTH_TEXELS * texel).xy * light_radius;

    vec2 depth2lm = min(depth2l, min(depth1l, depth3l));
    vec2 depth1lm = min(depth1l, min(depth2l, depth0 ));
    vec2 depth0m  = min(depth0 , min(depth1l, depth1r));
    vec2 depth1rm = min(depth1r, min(depth0 , depth2r));
    vec2 depth2rm = min(depth2r, min(depth1r, depth3r));

    vec2 vis_depth2lm = step(dist_to_light, depth2lm);
    vec2 vis_depth1lm = step(dist_to_light, depth1lm);
    vec2 vis_depth0m  = step(dist_to_light, depth0m );
    vec2 vis_depth1rm = step(dist_to_light, depth1rm);
    vec2 vis_depth2rm = step(dist_to_light, depth2rm);

    float inner_light = front_light *
        pow(1.0 - clamp((dist_to_light - depth0m.x) / params.back_glow, 0.0, 1.0), 4.0);

    vec2 vis_avg = (vis_depth2lm + vis_depth1lm + vis_depth0m + vis_depth1rm + vis_depth2rm) / 5.0;

    return vis_avg.x * front_light + (1.0 - vis_avg.x) * vis_avg.y * inner_light;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
flat in vec4 v_light_params;
flat in vec3 v_light_color;
flat in float v_light_offset;
in vec2 v_screen_uv;
in vec3 v_delta;
out vec4 f_color;

void main() {
    vec3 normal_value = texture(screen_normals, v_screen_uv).xyz;
    vec3 normal = normal_value * 2.0 - 1.0;

    float scale = normal_value == vec3(0.0) ?
        1.0 :
        max(dot(normalize(-v_delta), normalize(normal)), 0.0);

    vec3 color = v_light_color *
        scale *
        visibility(
            shadow_map,
            v_light_offset,
            v_light_params,
            v_delta.xy
        );
    f_color = vec4(color, 1.0);
}
"#;

impl ScreenLightPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: UNIFORM_BLOCKS,
            samplers: SAMPLERS,
            vertex_source: &VERTEX_SOURCE
                .replace("{max_num_lights}", &params.max_num_lights.to_string()),
            fragment_source: &format!(
                "{}\n{}",
                VISIBILITY_SOURCE,
                FRAGMENT_SOURCE.replace(
                    "{shadow_map_resolution}",
                    &params.shadow_map_resolution.to_string(),
                )
            ),
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw(
        &self,
        matrices: &Uniform<MatricesBlock>,
        params: &Uniform<GlobalLightParamsBlock>,
        shadow_map: &Texture,
        screen_normals: &Texture,
        draw_unit: DrawUnit<LightAreaVertex>,
    ) {
        gl::draw(
            &self.program,
            (matrices, params),
            [shadow_map, screen_normals],
            draw_unit,
            &DrawParams {
                blend: Some(Blend {
                    equation: BlendEquation::same(BlendOp::Add),
                    func: BlendFunc::same(BlendFactor::One, BlendFactor::One),
                    ..Blend::default()
                }),
                ..DrawParams::default()
            },
        )
    }
}
