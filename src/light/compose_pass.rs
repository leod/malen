use std::rc::Rc;

use nalgebra::{Point2, Vector2};

use crate::{
    data::{Mesh, Sprite, SpriteBatch, SpriteVertex},
    geom::Rect,
    gl::{self, DrawParams, Program, ProgramDef, Texture, Uniform},
};

use super::{def::GlobalLightParamsBlock, GLOBAL_LIGHT_PARAMS_BLOCK_BINDING};

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
    vec3 albedo = texture(screen_albedo, v_tex_coords).rgb;
    vec3 light = texture(screen_light, v_tex_coords).rgb;
    vec3 diffuse = albedo * (light + global_light_params.ambient);
    vec3 mapped = diffuse / (diffuse + vec3(1.0));

    const float gamma = 2.2;

    f_color = vec4(pow(mapped, vec3(1.0 / global_light_params.gamma)), 1.0);
    //f_color = vec4(diffuse, 1.0);
}
"#;

pub struct ComposePass {
    screen_rect: Mesh<SpriteVertex>,
    program: Program<GlobalLightParamsBlock, SpriteVertex, 2>,
}

impl ComposePass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let screen_rect = SpriteBatch::from_geometry(
            gl.clone(),
            Sprite {
                rect: Rect {
                    center: Point2::origin(),
                    size: Vector2::new(2.0, 2.0),
                },
                z: 0.0,
                tex_rect: Rect::from_top_left(Point2::origin(), Vector2::new(1.0, 1.0)),
            },
        )?
        .into_mesh();

        let program_def = ProgramDef {
            uniform_blocks: [("global_light_params", GLOBAL_LIGHT_PARAMS_BLOCK_BINDING)],
            samplers: ["screen_albedo", "screen_light"],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            screen_rect,
            program,
        })
    }

    pub fn draw(
        &self,
        global_light_params: &Uniform<GlobalLightParamsBlock>,
        screen_albedo: &Texture,
        screen_light: &Texture,
    ) {
        //#[cfg(feature = "coarse-prof")]
        //coarse_prof::profile!("light::ComposePass::draw");

        gl::draw(
            &self.program,
            global_light_params,
            [screen_albedo, screen_light],
            self.screen_rect.draw_unit(),
            &DrawParams::default(),
        );
    }
}
