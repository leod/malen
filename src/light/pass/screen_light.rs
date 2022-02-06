use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, DrawUnit, Glsl,
        Texture, Uniform,
    },
    glsl,
    pass::{ViewMatrices, MATRICES_BLOCK_BINDING},
    program,
};

use super::{
    super::{light_area::LightAreaVertex, GlobalLightProps, LightPipelineParams},
    GLOBAL_LIGHT_PROPS_BLOCK_BINDING,
};

program! {
    program ScreenLightProgram
    params {
        params: LightPipelineParams,
    }
    uniforms {
        matrices: ViewMatrices = MATRICES_BLOCK_BINDING,
        global_light_props: GlobalLightProps = GLOBAL_LIGHT_PROPS_BLOCK_BINDING,
    }
    samplers {
        shadow_map: Sampler2,
        screen_normals: Sampler2,
    }
    attributes {
        a: LightAreaVertex,
    }
    defines [
        pi => std::f32::consts::PI,
        depth_texels => "2.0f",
        max_num_lights => params.max_num_lights,
    ]
    includes [VISIBILITY_SOURCE]
    vertex glsl! {
        flat out float v_light_radius;
        flat out vec4 v_light_params;
        flat out vec3 v_light_color;
        flat out float v_light_offset;
        out vec2 v_screen_uv;
        out vec3 v_delta;

        void main() {
            v_light_radius = a_light_position.w;
            v_light_params = a_light_params;
            v_light_color = a_light_color;
            v_light_offset = (float(a_light_index) + 0.5) / float({{max_num_lights}});

            vec3 p = matrices.projection * matrices.view * vec3(a_position, 1.0);
            gl_Position = vec4(p.xy, 0.0, 1.0);
            v_screen_uv = p.xy * 0.5 + 0.5;
            v_delta = vec3(a_position.xy, 0.0) - a_light_position.xyz;
        }
    }
    fragment glsl! {
        flat in float v_light_radius;
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

            float visibility =
                visibility(
                    shadow_map,
                    v_light_offset,
                    v_light_radius,
                    v_light_params,
                    v_delta.xy
                );

            if (visibility == -1.0)
                discard;

            vec3 color = v_light_color * scale * visibility;
            f_color = vec4(color, 1.0);
        }
    }
}

const VISIBILITY_SOURCE: Glsl = glsl! {
    float visibility(
        in sampler2D shadow_map,
        in float light_offset,
        in float light_radius,
        in vec4 light_params,
        in vec2 delta
    ) {
        float light_angle = light_params.x;
        float light_angle_size = light_params.y;
        float light_start = light_params.z;
        float light_back_glow = light_params.w;

        float dist_to_light = length(delta);
        if (dist_to_light > light_radius)
            return -1.0;

        float angle = atan(delta.y, delta.x);

        float fall_on = (1.0 + sin({{pi}} * (3.0/2.0 +
            clamp(dist_to_light / light_start - 1.0, 0.0, 1.0)))) / 2.0;
        float front_light = fall_on * pow(1.0 - dist_to_light / light_radius, 2.0);
        float angle_diff = mod(abs(angle - light_angle), 2.0 * {{pi}});
        if (angle_diff > {{pi}})
            angle_diff = 2.0 * {{pi}} - angle_diff;

        float angle_to_border = angle_diff * 2.0 - light_angle_size
            + global_light_props.angle_fall_off_size;
        if (abs(light_angle_size - 2.0 * {{pi}}) > 0.001 && angle_to_border > 0.0) {
            float t = angle_to_border / global_light_props.angle_fall_off_size;
            front_light *= 2.0 / (1.0 + 1.0 * exp(10.0 * t));
        }

        vec2 c = vec2(angle / (2.0 * {{pi}}) + 0.5, light_offset);
        vec2 texel = vec2(1.0 / float(textureSize(shadow_map, 0).x), 0.0);

        vec2 depth3l = texture(shadow_map, c - 3.0 * {{depth_texels}} * texel).xy * light_radius;
        vec2 depth2l = texture(shadow_map, c - 2.0 * {{depth_texels}} * texel).xy * light_radius;
        vec2 depth1l = texture(shadow_map, c - 1.0 * {{depth_texels}} * texel).xy * light_radius;
        vec2 depth0  = texture(shadow_map, c                                 ).xy * light_radius;
        vec2 depth1r = texture(shadow_map, c + 1.0 * {{depth_texels}} * texel).xy * light_radius;
        vec2 depth2r = texture(shadow_map, c + 2.0 * {{depth_texels}} * texel).xy * light_radius;
        vec2 depth3r = texture(shadow_map, c + 3.0 * {{depth_texels}} * texel).xy * light_radius;

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
            pow(1.0 - clamp((dist_to_light - depth0m.x) / light_back_glow, 0.0, 1.0), 4.0);

        vec2 vis_avg = (vis_depth2lm + vis_depth1lm + vis_depth0m + vis_depth1rm + vis_depth2rm) / 5.0;

        return vis_avg.x * front_light + (1.0 - vis_avg.x) * vis_avg.y * inner_light;
    }
};

pub struct ScreenLightPass {
    program: ScreenLightProgram,
}

impl ScreenLightPass {
    pub fn new(gl: Rc<gl::Context>, params: LightPipelineParams) -> Result<Self, gl::Error> {
        let program = ScreenLightProgram::new(gl, params)?;

        Ok(Self { program })
    }

    pub fn draw(
        &self,
        matrices: &Uniform<ViewMatrices>,
        global_light_props: &Uniform<GlobalLightProps>,
        shadow_map: &Texture,
        screen_normals: &Texture,
        draw_unit: DrawUnit<LightAreaVertex>,
    ) {
        gl::draw(
            &self.program,
            (matrices, global_light_props),
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
