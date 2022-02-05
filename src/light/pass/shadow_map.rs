use std::rc::Rc;

use crate::{
    gl::{
        self, Blend, BlendEquation, BlendFactor, BlendFunc, BlendOp, DrawParams, InstancedDrawUnit,
    },
    program,
};

use super::super::{Light, OccluderLineVertex};

program! {
    |max_num_lights: u32|
        Program [
            (),
            (),
            (a_occluder: OccluderLineVertex, i_light: Light)
        ]
        => (
            &VERTEX_SOURCE.replace("{max_num_lights}", &max_num_lights.to_string()),
            FRAGMENT_SOURCE,
        )
}

const VERTEX_SOURCE: &str = r#"
flat out vec2 v_light_position;
flat out float v_light_radius;
flat out int v_is_front;
out vec4 v_edge;
out float v_angle;

float angle_to_light(vec2 position) {
    vec2 delta = position - i_light_position.xy;
    return atan(delta.y, delta.x);
}

const float PI = 3.141592;

void main() {
    if (gl_InstanceID == a_occluder_ignore_light_index1
            || gl_InstanceID == a_occluder_ignore_light_index2
            || i_light_position.z >= a_occluder_height) {
        gl_Position = vec4(-10.0, -10.0, -10.0, 1.0);
        return;
    }

    v_light_position = i_light_position.xy;
    v_light_radius = i_light_radius;

    vec3 c = cross(vec3(a_occluder_line_0 - i_light_position.xy, 0.0),
                   vec3(a_occluder_line_1 - i_light_position.xy, 0.0));
    v_is_front = (((a_occluder_order == 0 || a_occluder_order == 2) && c.z < 0.0) ||
                  ((a_occluder_order == 1 || a_occluder_order == 3) && c.z > 0.0))
                 ? 1 : 0;

    float angle_0 = angle_to_light(a_occluder_line_0);
    float angle_1 = angle_to_light(a_occluder_line_1);

    v_edge = vec4(a_occluder_line_0, a_occluder_line_1);
    v_edge = mix(v_edge, v_edge.zwxy, step(angle_0, angle_1));
    v_angle = angle_0;
    if (abs(angle_0 - angle_1) > PI) {
        if (a_occluder_order == 0) {
            v_angle = -PI;
        } else if (a_occluder_order == 1 || a_occluder_order == 2) {
            v_angle = min(angle_0, angle_1);
        } else {
            v_angle = PI;
        }
    }

    gl_Position = vec4(
        v_angle / PI,
        (float(gl_InstanceID) + 0.5) / float({max_num_lights}) * 2.0 - 1.0,
        0.0,
        1.0
    );
}
"#;

const FRAGMENT_SOURCE: &str = r#"
flat in vec2 v_light_position;
flat in float v_light_radius;
flat in int v_is_front;
in vec4 v_edge;
in float v_angle;
out vec4 f_color;

float ray_line_segment_intersection(
    vec2 o,
    vec2 d,
    vec2 p,
    vec2 q
) {
    /**
        ray(s) = o + d * s             (0 <= s)
        line(t) = p + (q - p) * t      (0 <= t <= 1)
    
        ray(s) = line(t)
            <=> o + d * s = p + (q - p) * t
            <=> d * s + (p - q) * t = p - o
            <=> M * [[s], [t]] = p - o
              where M = [[d.x, d.y], [p.x - q.x, p.y - q.y]] 
            <=> [[s], [t]] = M^-1 (p - o)   (if M is invertible)
    **/

    float det = d.x * (p.y - q.y) + d.y * (q.x - p.x);
    if (abs(det) < 0.0000001)
        return 1.0;

    mat2 m = mat2(d.x, d.y, p.x - q.x, p.y - q.y);
    vec2 time = inverse(m) * (p - o);

    float s = time.x;
    float t = time.y;
    if (s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0) {
        return s;
    } else {
        return 1.0;
    }
}

void main() {
    float t = ray_line_segment_intersection(
        v_light_position,
        vec2(cos(v_angle), sin(v_angle)) * v_light_radius,
        v_edge.xy,
        v_edge.zw
    );
    f_color = vec4(
        v_is_front == 0 ? vec2(1.0, t)
                        : vec2(t, 1.0),
        0.0, 1.0);
}
"#;

pub struct ShadowMapPass {
    program: Program,
}

impl ShadowMapPass {
    pub fn new(gl: Rc<gl::Context>, max_num_lights: u32) -> Result<Self, gl::Error> {
        let program = Program::new(gl, max_num_lights)?;

        Ok(Self { program })
    }

    pub fn draw(&self, draw_unit: InstancedDrawUnit<(OccluderLineVertex, Light)>) {
        gl::draw_instanced(
            &self.program,
            (),
            [],
            draw_unit,
            &DrawParams {
                blend: Some(Blend {
                    equation: BlendEquation::same(BlendOp::Min),
                    func: BlendFunc::same(BlendFactor::One, BlendFactor::One),
                    ..Blend::default()
                }),
                ..DrawParams::default()
            },
        )
    }
}
