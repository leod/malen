use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Program,
        ProgramDef, Texture, UniformBuffer,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{light_area::LightAreaVertex, LightPipelineParams};

pub(super) const VISIBILITY_SOURCE: &str = r#"
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

    vec2 tex_coords = vec2(angle / (2.0 * PI) + 0.5, light_offset);
    vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);

    float front1 = texture(shadow_map, tex_coords).r * light_radius;
    float front2 = texture(shadow_map, tex_coords - 1.0 * texel).r * light_radius;
    float front3 = texture(shadow_map, tex_coords + 1.0 * texel).r * light_radius;

    float back1 = texture(shadow_map, tex_coords).g * light_radius;
    float back2 = texture(shadow_map, tex_coords - 1.0 * texel).g * light_radius;
    float back3 = texture(shadow_map, tex_coords + 1.0 * texel).g * light_radius;

    float front_glow = 15.0;
    float back_glow = 20.0;
    float front = min(min(front1, front2), front3) - front_glow;
    float back = min(min(back1, back2), back3);

    //float front = 0.5 * front1 + 0.25 * front2 + 0.25 * front3 - front_glow;
    //float back = 0.5 * back1 + 0.25 * back2 + 0.25 * back3;

    float v_front = step(dist_to_light, front);
    float v_back = step(dist_to_light, back);

    float fall_on = (1.0 + sin(PI * (3.0/2.0 +
        clamp(dist_to_light / light_start - 1.0, 0.0, 1.0)))) / 2.0;

    float front_light = fall_on * pow(1.0 - dist_to_light / light_radius, 2.0);
    //float front_light = fall_on * pow(1.0 / (dist_to_light/50.0), 2.0);

    float angle_diff = mod(abs(angle - light_angle), 2.0 * PI);
    if (angle_diff > PI)
        angle_diff = 2.0 * PI - angle_diff;

    float angle_fall_off_size = PI / 20.0;
    if (abs(light_angle_size - 2.0 * PI) > 0.001
            && angle_diff * 2.0 > light_angle_size - angle_fall_off_size) {
        float t = (angle_diff * 2.0 - light_angle_size + angle_fall_off_size) / angle_fall_off_size;
        front_light *= 2.0 / (1.0 + 1.0 * exp(10.0 * t));
    }

    float inner_light = front_light;
    float to_front = dist_to_light - front;
    if (to_front < front_glow) {
        inner_light *= 2.0 + sin(PI * (3.0/2.0 + to_front / front_glow));
    } else {
        float glow_size = 40.0; //min(40.0, back - front);
        inner_light *= 2.0 * pow(
            1.0 - clamp((to_front - front_glow) / glow_size, 0.0, 1.0),
            5.0);
        //inner_light *= 2.0 * max(0.0, 1.0 - (to_front - front_glow) / glow_size);
    } 

    return front_light * v_front + inner_light * (1.0 - v_front) * v_back;
}
"#;

const VERTEX_SOURCE: &str = r#"
flat out vec4 v_light_params;
flat out vec3 v_light_color;
flat out float v_light_offset;
out vec2 v_screen_pos;
out vec2 v_delta;

void main() {
    v_light_params = a_light_params;
    v_light_color = a_light_color;
    v_light_offset = (float(a_light_index) + 0.5) / float({max_num_lights});

    vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_screen_pos = p.xy;
    v_delta = a_position.xy - a_light_position;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
flat in vec4 v_light_params;
flat in vec3 v_light_color;
flat in float v_light_offset;
in vec2 v_screen_pos;
in vec2 v_delta;
out vec4 f_color;

void main() {
    vec3 normal_value = texture(screen_normals, v_screen_pos * 0.5 + 0.5).xyz;
    vec3 normal = normal_value * 2.0 - 1.0;
    normal.y = -normal.y;

    float scale = normal_value == vec3(0.0) ?
        1.0 :
        max(dot(normalize(vec3(-v_delta, 0.0) + vec3(0, 0, 50)), normalize(normal)), 0.0);

    vec3 color = v_light_color *
        scale *
        visibility(
            shadow_map,
            v_light_offset,
            v_light_params,
            v_delta
        );
    f_color = vec4(color, 1.0);
}
"#;

pub struct ScreenLightPass {
    program: Program<MatricesBlock, LightAreaVertex, 2>,
}

impl ScreenLightPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [("matrices", MATRICES_BLOCK_BINDING)],
            samplers: ["shadow_map", "screen_normals"],
            vertex_source: &VERTEX_SOURCE
                .replace("{max_num_lights}", &params.max_num_lights.to_string()),
            fragment_source: &format!(
                "{}\n{}",
                VISIBILITY_SOURCE,
                FRAGMENT_SOURCE.replace(
                    "{shadow_map_resolution}",
                    &params.shadow_map_resolution.to_string(),
                ),
            ),
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self { program })
    }

    pub fn draw(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        shadow_map: &Texture,
        screen_normals: &Texture,
        draw_unit: DrawUnit<LightAreaVertex>,
    ) {
        //#[cfg(feature = "coarse-prof")]
        //coarse_prof::profile!("light::ScreenLightPass::draw");

        gl::draw(
            &self.program,
            matrices,
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
