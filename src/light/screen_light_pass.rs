use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Program,
        ProgramDef, Texture, UniformBuffer,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{
    light_area::LightAreaVertex, GlobalLightParamsBlock, LightPipelineParams,
    GLOBAL_LIGHT_PARAMS_BLOCK_BINDING,
};

const VERTEX_SOURCE: &str = r#"
flat out vec4 v_light_params;
flat out vec3 v_light_color;
flat out float v_light_offset;
out vec2 v_screen_pos;
out vec3 v_delta;

void main() {
    v_light_params = a_light_params;
    v_light_color = a_light_color;
    v_light_offset = (float(a_light_index) + 0.5) / float({max_num_lights});

    vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_screen_pos = p.xy;
    v_delta = vec3(a_position.xy, 0.0) - a_light_position;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
float visibility(
    in sampler2D shadow_map,
    in float light_offset,
    in vec4 light_params,
    in vec2 delta
) {
    const float PI = 3.141592;

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

    if (abs(light_angle_size - 2.0 * PI) > 0.001
            && angle_diff * 2.0 > light_angle_size - global_light_params.angle_fall_off_size) {
        float t = (
            angle_diff * 2.0
            - light_angle_size
            + global_light_params.angle_fall_off_size
        ) / global_light_params.angle_fall_off_size;
        front_light *= 2.0 / (1.0 + 1.0 * exp(10.0 * t));
    }

    vec2 tex_coords = vec2(angle / (2.0 * PI) + 0.5, light_offset);
    vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);

    float front3l = texture(shadow_map, tex_coords - 6.0 * texel).r * light_radius;
    float front2l = texture(shadow_map, tex_coords - 4.0 * texel).r * light_radius;
    float front1l = texture(shadow_map, tex_coords - 2.0 * texel).r * light_radius;
    float front0 = texture(shadow_map, tex_coords).r * light_radius;
    float front1r = texture(shadow_map, tex_coords + 2.0 * texel).r * light_radius;
    float front2r = texture(shadow_map, tex_coords + 4.0 * texel).r * light_radius;
    float front3r = texture(shadow_map, tex_coords + 6.0 * texel).r * light_radius;

    float back3l = texture(shadow_map, tex_coords - 6.0 * texel).g * light_radius;
    float back2l = texture(shadow_map, tex_coords - 4.0 * texel).g * light_radius;
    float back1l = texture(shadow_map, tex_coords - 2.0 * texel).g * light_radius;
    float back0 = texture(shadow_map, tex_coords).g * light_radius;
    float back1r = texture(shadow_map, tex_coords + 2.0 * texel).g * light_radius;
    float back2r = texture(shadow_map, tex_coords + 4.0 * texel).g * light_radius;
    float back3r = texture(shadow_map, tex_coords + 6.0 * texel).g * light_radius;

    float front0m = min(min(front1l, front1r), front0) - global_light_params.front_glow;
    float back0m = min(min(back1l, back1r), back0);

    float inner_light = front_light;
    float to_front = dist_to_light - front0m;
    if (to_front < global_light_params.front_glow) {
        inner_light *= 2.0 + sin(PI * (3.0/2.0 + to_front / global_light_params.front_glow));
    } else {
        inner_light *= 2.0 * pow(
            1.0 - clamp((to_front - global_light_params.front_glow) / global_light_params.back_glow, 0.0, 1.0),
            4.0);
    } 

    float front2lm = min(min(front3l, front1l), front2l) - global_light_params.front_glow;
    float front1lm = min(min(front2l, front0), front1l) - global_light_params.front_glow;
    float front1rm = min(min(front2r, front0), front1r) - global_light_params.front_glow;
    float front2rm = min(min(front3r, front1r), front2r) - global_light_params.front_glow;
    float back2lm = min(min(back3l, back1l), back2l);
    float back1lm = min(min(back2l, back0), back1l);
    float back1rm = min(min(back2r, back0), back1r);
    float back2rm = min(min(back3r, back1r), back2r);

    float vis_front2lm = step(dist_to_light, front2lm);
    float vis_front1lm = step(dist_to_light, front1lm);
    float vis_front0m = step(dist_to_light, front0m);
    float vis_front1rm = step(dist_to_light, front1rm);
    float vis_front2rm = step(dist_to_light, front2rm);

    float vis_back2lm = step(dist_to_light, back2lm);
    float vis_back1lm = step(dist_to_light, back1lm);
    float vis_back0m = step(dist_to_light, back0m);
    float vis_back1rm = step(dist_to_light, back1rm);
    float vis_back2rm = step(dist_to_light, back2rm);

    //float vis_front = vis_front0m;
    //float vis_back = vis_back0m;
    float vis_front = (vis_front0m + vis_front1lm + vis_front2lm + vis_front1rm + vis_front2rm) / 5.0;
    float vis_back = (vis_back0m + vis_back1lm + vis_back2lm + vis_back1rm + vis_back2rm) / 5.0;

    return front_light * vis_front + inner_light * (1.0 - vis_front) * vis_back;
}

flat in vec4 v_light_params;
flat in vec3 v_light_color;
flat in float v_light_offset;
in vec2 v_screen_pos;
in vec3 v_delta;
out vec4 f_color;

void main() {
    vec3 normal_value = texture(screen_normals, v_screen_pos * 0.5 + 0.5).xyz;
    vec3 normal = normal_value * 2.0 - 1.0;
    normal.y = -normal.y;

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

pub struct ScreenLightPass {
    program: Program<(MatricesBlock, GlobalLightParamsBlock), LightAreaVertex, 2>,
}

impl ScreenLightPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("global_light_params", GLOBAL_LIGHT_PARAMS_BLOCK_BINDING),
            ],
            samplers: ["shadow_map", "screen_normals"],
            vertex_source: &VERTEX_SOURCE
                .replace("{max_num_lights}", &params.max_num_lights.to_string()),
            fragment_source: &FRAGMENT_SOURCE.replace(
                "{shadow_map_resolution}",
                &params.shadow_map_resolution.to_string(),
            ),
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        global_light_params: &UniformBuffer<GlobalLightParamsBlock>,
        shadow_map: &Texture,
        screen_normals: &Texture,
        draw_unit: DrawUnit<LightAreaVertex>,
    ) {
        //#[cfg(feature = "coarse-prof")]
        //coarse_prof::profile!("light::ScreenLightPass::draw");

        gl::draw(
            &self.program,
            (matrices, global_light_params),
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
