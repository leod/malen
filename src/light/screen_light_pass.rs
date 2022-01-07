use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Program,
        ProgramDef, Texture, UniformBuffer,
    },
    pass::{MatricesBlock, MATRICES_BLOCK_BINDING},
};

use super::data::LightAreaVertex;

const VERTEX_SOURCE: &str = r#"
out vec2 v_delta;
out vec3 v_light_params;
out vec3 v_light_color;
out float v_light_offset;

void main() {
    vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
    gl_Position = vec4(p.xy, 0.0, 1.0);

    v_delta = a_position.xy - a_light_position;
    v_light_params = a_light_params;
    v_light_color = a_light_color;
    v_light_offset = (float(a_light_index) + 0.5) / float({max_num_lights});
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_delta;
in vec3 v_light_params;
in vec3 v_light_color;
in float v_light_offset;
out vec4 f_color;

const float PI = 3.141592;

void main() {
    float angle = atan(v_delta.y, v_delta.x);
    float dist_to_light = length(v_delta);
    float light_radius = v_light_params.x;

    vec2 tex_coords = vec2(angle / (2.0 * PI) + 0.5, v_light_offset);
    //vec2 texel = vec2(1.0 / float({shadow_map_resolution}), 0.0);
    vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);
    float dist1 = texture(shadow_map, tex_coords).r * light_radius;
    float dist2 = texture(shadow_map, tex_coords - 1.0 * texel).r * light_radius;
    float dist3 = texture(shadow_map, tex_coords + 1.0 * texel).r * light_radius;

    float vis1 = step(dist_to_light, dist1);
    float vis2 = step(dist_to_light, dist2);
    float vis3 = step(dist_to_light, dist3);

    //float visibility = step(dist_to_light, dist1);
    float visibility = max(vis1, max(vis2, vis3));
    /*visibility *= 0.5;
    visibility += step(dist_to_light, dist2) * 0.25;
    visibility += step(dist_to_light, dist3) * 0.25;*/

    visibility *= pow(1.0 - dist_to_light / light_radius, 2.0);

    float angle_diff = mod(abs(angle - v_light_params.y), 2.0 * PI);
    if (angle_diff > PI)
        angle_diff = 2.0 * PI - angle_diff;
    //visibility *= pow(exp(1.0 - clamp(angle_diff / v_light_params.z, 0.0, 1.0)), 0.5); 
    //visibility *= pow(1.0 - clamp(angle_diff / v_light_params.z, 0.0, 1.0), 0.1); 
    visibility *= step(angle_diff, v_light_params.z);

    vec3 color = v_light_color * visibility;
    f_color = vec4(color, 1.0);
}
"#;

pub struct ScreenLightPass {
    program: Program<MatricesBlock, LightAreaVertex, 1>,
}

impl ScreenLightPass {
    pub fn new(
        gl: Rc<gl::Context>,
        shadow_map_resolution: u32,
        max_num_lights: u32,
    ) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [("matrices", MATRICES_BLOCK_BINDING)],
            samplers: ["shadow_map"],
            vertex_source: &VERTEX_SOURCE.replace("{max_num_lights}", &max_num_lights.to_string()),
            fragment_source: &FRAGMENT_SOURCE.replace(
                "{shadow_map_resolution}",
                &shadow_map_resolution.to_string(),
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
