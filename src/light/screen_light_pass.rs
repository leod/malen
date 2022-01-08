use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Program,
        ProgramDef, Texture, UniformBuffer,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{data::LightAreaVertex, LightPipelineParams};

pub(super) const VISIBILITY_SOURCE: &str = r#"
float visibility(
    in sampler2D shadow_map,
    in float light_offset,
    in vec3 light_params,
    in vec2 delta
) {
    const float PI = 3.141592;

    float light_radius = light_params.x;
    float light_angle = light_params.y;
    float light_angle_size = light_params.z;

    float angle = atan(delta.y, delta.x);
    float dist_to_light = length(delta);

    vec2 tex_coords = vec2(angle / (2.0 * PI) + 0.5, light_offset);
    vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);

    float dist1 = texture(shadow_map, tex_coords).r * light_radius;
    float dist2 = texture(shadow_map, tex_coords - 1.0 * texel).r * light_radius;
    float dist3 = texture(shadow_map, tex_coords + 1.0 * texel).r * light_radius;

    float vis1 = step(dist_to_light, dist1);
    float vis2 = step(dist_to_light, dist2);
    float vis3 = step(dist_to_light, dist3);

    //float v = max(vis1, max(vis2, vis3));
    float v = min(vis1, min(vis2, vis3));
    //float v = 0.5 * vis1 + 0.25 * vis2 + 0.25 * vis3;

    v *= pow(1.0 - dist_to_light / light_radius, 2.0);

    float angle_diff = mod(abs(angle - light_angle), 2.0 * PI);
    if (angle_diff > PI)
        angle_diff = 2.0 * PI - angle_diff;
    float angle_tau = clamp(2.0 * angle_diff / light_angle_size, 0.0, 1.0);

    v *= pow(1.0 - angle_tau, 0.2); 
    v *= step(angle_tau, light_angle_size);

    return v;
}
"#;

const VERTEX_SOURCE: &str = r#"
flat out vec3 v_light_params;
flat out vec3 v_light_color;
flat out float v_light_offset;
out vec2 v_delta;

void main() {
    v_light_params = a_light_params;
    v_light_color = a_light_color;
    v_light_offset = (float(a_light_index) + 0.5) / float({max_num_lights});

    vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_delta = a_position.xy - a_light_position;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
flat in vec3 v_light_params;
flat in vec3 v_light_color;
flat in float v_light_offset;
in vec2 v_delta;
out vec4 f_color;

void main() {
    vec3 color = v_light_color *
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
    program: Program<MatricesBlock, LightAreaVertex, 1>,
}

impl ScreenLightPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [("matrices", MATRICES_BLOCK_BINDING)],
            samplers: ["shadow_map"],
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
        draw_unit: DrawUnit<LightAreaVertex>,
    ) {
        gl::draw(
            &self.program,
            matrices,
            [shadow_map],
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
