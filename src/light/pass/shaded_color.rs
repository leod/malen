use std::rc::Rc;

use crate::{
    data::ColorVertex,
    gl::{self, DrawParams, DrawUnit, Element, Texture, Uniform},
    pass::{ViewMatrices, MATRICES_BLOCK_BINDING},
    program,
};

program! {
    program ShadedColorProgram
    uniforms {
        matrices: ViewMatrices = MATRICES_BLOCK_BINDING,
    }
    samplers {
        screen_light: Sampler2,
    }
    attributes {
        a: ColorVertex,
    }
    vertex glsl! {
        out vec3 v_color;
        out vec2 v_screen_uv;

        void main() {
            vec3 position = matrices.projection
                * matrices.view
                * vec3(a_position.xy, 1.0);

            gl_Position = vec4(position.xy, a_position.z, 1.0);
            v_color = pow(vec3(a_color), vec3(2.2));
            v_screen_uv = vec2(position.xy) * 0.5 + 0.5;
        }
    }
    fragment glsl! {
        in vec3 v_color;
        in vec2 v_screen_uv;
        out vec4 f_color;

        void main() {
            vec3 light = texture(screen_light, v_screen_uv).rgb;
            f_color = vec4(v_color * light, 1.0);
        }
    }
}

pub struct ShadedColorPass {
    program: ShadedColorProgram,
}

impl ShadedColorPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program = ShadedColorProgram::new(gl)?;

        Ok(Self { program })
    }

    pub fn draw<E>(
        &self,
        matrices: &Uniform<ViewMatrices>,
        screen_light: &Texture,
        draw_unit: DrawUnit<ColorVertex, E>,
        draw_params: &DrawParams,
    ) where
        E: Element,
    {
        gl::draw(
            &self.program,
            matrices,
            [screen_light],
            draw_unit,
            draw_params,
        );
    }
}
