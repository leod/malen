use std::rc::Rc;

use crevice::{glsl::GlslStruct, std140::AsStd140};
use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Texture, Uniform, UniformBlock},
    program, Color4,
};

use super::BLUR_PROPS_BLOCK_BINDING;

#[derive(Default, Debug, Copy, Clone, AsStd140, GlslStruct)]
struct BlurProps {
    dir: u32,
}

impl UniformBlock for BlurProps {}

program! {
    BlurProgram [
        blur_props: BlurProps = BLUR_PROPS_BLOCK_BINDING;
        input: Sampler2;
        a: SpriteVertex,
    ]
    -> (VERTEX_SOURCE, FRAGMENT_SOURCE)
}

const VERTEX_SOURCE: &str = r#"
out vec2 v_tex_coords;
void main() {
    gl_Position = vec4(a_position.xyz, 1.0);
    v_tex_coords = a_tex_coords;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_tex_coords;
out vec4 f_color;
void main() {
uniform sampler2D image;
  
uniform bool horizontal;
uniform float weight[5] = float[] (0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216);

void main()
{             
    vec2 tex_offset = 1.0 / textureSize(image, 0); // gets size of single texel
    vec3 result = texture(image, TexCoords).rgb * weight[0]; // current fragment's contribution
    if(horizontal)
    {
        for(int i = 1; i < 5; ++i)
        {
            result += texture(image, TexCoords + vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
            result += texture(image, TexCoords - vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
        }
    }
    else
    {
        for(int i = 1; i < 5; ++i)
        {
            result += texture(image, TexCoords + vec2(0.0, tex_offset.y * i)).rgb * weight[i];
            result += texture(image, TexCoords - vec2(0.0, tex_offset.y * i)).rgb * weight[i];
        }
    }
    FragColor = vec4(result, 1.0);
}
}
"#;

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
