use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Texture, Uniform},
    program, Color4,
};

use super::{super::GlobalLightProps, GLOBAL_LIGHT_PROPS_BLOCK_BINDING};

program! {
    program ComposeProgram
    uniforms {
        global_light_props: GlobalLightProps = GLOBAL_LIGHT_PROPS_BLOCK_BINDING,
    }
    samplers {
        screen_albedo: Sampler2,
        screen_light: Sampler2,
    }
    attributes {
        a: SpriteVertex,
    }
    vertex glsl! {
        out vec2 v_tex_coords;
        void main() {
            gl_Position = vec4(a_position.xyz, 1.0);
            v_tex_coords = a_tex_coords;
        }
    }
    fragment glsl! {
        in vec2 v_tex_coords;
        out vec4 f_color;
        void main() {
            vec4 albedo = texture(screen_albedo, v_tex_coords);
            vec3 light = texture(screen_light, v_tex_coords).rgb;
            vec3 diffuse = vec3(albedo) * (light + global_light_props.ambient);
            vec3 mapped = diffuse / (diffuse + vec3(1.0));
            f_color = vec4(pow(mapped, vec3(1.0 / global_light_props.gamma)), 1.0);
        }
    }
}

// => struct ComposeProgram(Program<GlobalLightProps, SpriteVertex, 2>)

pub struct ComposePass {
    screen_rect: Mesh<SpriteVertex>,
    program: ComposeProgram,
}

impl ComposePass {
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

        let program = ComposeProgram::new(gl)?;

        Ok(Self {
            screen_rect,
            program,
        })
    }

    pub fn draw(
        &self,
        global_light_props: &Uniform<GlobalLightProps>,
        screen_albedo: &Texture,
        screen_light: &Texture,
    ) {
        gl::draw(
            &self.program,
            global_light_props,
            [screen_albedo, screen_light],
            self.screen_rect.draw_unit(),
            &DrawParams::default(),
        );
    }
}
