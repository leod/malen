use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, InstancedDrawUnit,
        Program, ProgramDef, Texture, UniformBuffer,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::{Light, LightPipelineParams, OccluderLineVertex};

pub(super) const VISIBILITY_SOURCE: &str = r#"
float visibility(
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

    float vis1 = step(dist_to_light, dist1 + 1.0);
    float vis2 = step(dist_to_light, dist2);
    float vis3 = step(dist_to_light, dist3);

    float v = max(vis1, max(vis2, vis3));
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
flat out vec2 v_light_position;
flat out vec3 v_line_color;
flat out vec2 v_line_normal;
out vec2 v_line_position;

void main() {
    if (gl_InstanceID == a_ignore_light_index || a_order >= 2) {
        gl_Position = vec4(-10.0, -10.0, -10.0, -10.0);
        return;
    }

    v_light_params = vec3(i_light_radius, i_light_angle, i_light_angle_size);
    v_light_color = i_light_color;
    v_light_offset = (float(gl_InstanceID) + 0.5) / float({max_num_lights});
    v_light_position = i_light_position;

    vec2 s = a_line_0; 
    /*if (a_order == 0)
        s -= 2.0 * normalize(a_line_1 - a_line_0);
    else
        s += 2.0 * normalize(a_line_1 - a_line_0);*/

    vec3 p = matrices.projection * matrices.view * vec3(s, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);
    v_line_position = a_line_0;

    vec2 edge = a_order == 0 ? a_line_0 - a_line_1 : a_line_1 - a_line_0;

    v_line_color = a_color;
    v_line_normal = normalize(vec2(edge.y, -edge.x));
}
"#;

const FRAGMENT_SOURCE: &str = r#"
flat in vec3 v_light_params;
flat in vec3 v_light_color;
flat in float v_light_offset;
flat in vec2 v_light_position;
flat in vec3 v_line_color;
flat in vec2 v_line_normal;
in vec2 v_line_position;
out vec4 f_color;

void main() {
    vec2 delta = v_line_position - v_light_position;
    float v = visibility(
        v_light_offset,
        v_light_params,
        delta
    );

    float is_non_orth = abs(dot(v_line_normal, normalize(delta))) < 0.1 ? 0.0: 1.0;

    vec3 color = v * is_non_orth * 3.0 * max(dot(v_line_normal, normalize(delta)), 0.0) * v_line_color * v_light_color;
    f_color = vec4(color, 1.0);
}
"#;

pub struct ScreenGlowPass {
    program: Program<MatricesBlock, (OccluderLineVertex, Light), 1>,
}

impl ScreenGlowPass {
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
        draw_unit: InstancedDrawUnit<(OccluderLineVertex, Light)>,
    ) {
        gl::draw_instanced(
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
                line_width: 5.0,
                ..DrawParams::default()
            },
        )
    }
}
