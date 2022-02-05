use std::rc::Rc;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Framebuffer, Texture, Uniform, UniformBlock},
    program, Color4,
};

use super::BLUR_PROPS_BLOCK_BINDING;

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
pub struct BlurProps {
    direction: u32,
}

impl UniformBlock for BlurProps {}

program! {
    BlurProgram[
        blur_props: BlurProps = BLUR_PROPS_BLOCK_BINDING;
        input: Sampler2;
        a: SpriteVertex,
    ]
    => (VERTEX_SOURCE, FRAGMENT_SOURCE)
}

const VERTEX_SOURCE: &str = r#"
out vec2 v_uv;
void main() {
    gl_Position = vec4(a_position.xyz, 1.0);
    v_uv = a_tex_coords;
}
"#;

// https://learnopengl.com/Advanced-Lighting/Bloom
const FRAGMENT_SOURCE: &str = r#"
in vec2 v_uv;
out vec4 f_color;
  
const float weight[{num_weights}] = float[] ({weights});

void main() {             
    vec2 texel = 1.0 / textureSize(input, 0);

    vec3 result = texture(input, v_uv).rgb * weight[0];

    if (blur_props.direction == 0) {
        for (int i = 1; i < {num_weights}; i++) {
            result += texture(input, v_uv + vec2(texel.x * i, 0.0)).rgb * weight[i];
            result += texture(input, v_uv - vec2(texel.x * i, 0.0)).rgb * weight[i];
        }
    } else {
        for (int i = 1; i < {num_weights}; i++) {
            result += texture(input, v_uv + vec2(0.0, texel.y * i)).rgb * weight[i];
            result += texture(input, v_uv - vec2(0.0, texel.y * i)).rgb * weight[i];
        }
    }

    f_color = vec4(result, 1.0);
}
}
"#;

pub struct BlurBuffer {
    buffer: Framebuffer,
}

pub struct BlurPass {
    screen_rect: Mesh<SpriteVertex>,
    program: BlurProgram,
}

impl BlurPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
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

        let program = BlurProgram::new(gl)?;

        Ok(Self {
            screen_rect,
            program,
        })
    }

    pub fn draw(&self, input: &Texture) {
        /*gl::draw(
            &self.program,
            params,
            [input],
            self.screen_rect.draw_unit(),
            &DrawParams::default(),
        );*/
    }
}
