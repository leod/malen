use std::{cell::RefCell, rc::Rc};

use crate::{
    data::SpriteVertex,
    gl::{
        self, DepthTest, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, UniformBuffer,
    },
    pass::{
        MatricesBlock, SpriteInfoBlock, SpriteInfos, MATRICES_BLOCK_BINDING,
        SPRITE_INFO_BLOCK_BINDING,
    },
};

const VERTEX_SOURCE: &str = r#"
out vec2 v_uv;

void main() {
    vec3 position = matrices.projection
        * matrices.view
        * vec3(a_position.xy, 1.0);

    gl_Position = vec4(position.xy, a_position.z, 1.0);

    v_uv = a_tex_coords / sprite_info.size;
}
"#;

const FRAGMENT_SOURCE: &str = r#"
in vec2 v_uv;
layout (location = 0) out vec4 f_albedo;
layout (location = 1) out vec4 f_normal;

void main() {
    f_albedo = texture(sprite, v_uv);
    f_normal = texture(normal_map, v_uv);
}
"#;

pub struct GeometrySpriteNormalPass {
    program: Program<(MatricesBlock, SpriteInfoBlock), SpriteVertex, 2>,
    sprite_infos: RefCell<SpriteInfos>,
}

impl GeometrySpriteNormalPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("sprite_info", SPRITE_INFO_BLOCK_BINDING),
            ],
            samplers: ["sprite", "normal_map"],
            vertex_source: VERTEX_SOURCE,
            fragment_source: FRAGMENT_SOURCE,
        };
        let program = Program::new(gl, program_def)?;

        Ok(Self {
            program,
            sprite_infos: RefCell::new(SpriteInfos::new()),
        })
    }

    pub fn draw<E>(
        &self,
        matrices: &UniformBuffer<MatricesBlock>,
        texture: &Texture,
        normal_map: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        #[cfg(feature = "coarse-prof")]
        coarse_prof::profile!("light::GeometrySpriteNormalPass::draw");

        let mut sprite_infos = self.sprite_infos.borrow_mut();
        let sprite_info = sprite_infos.get(texture)?;

        gl::draw(
            &self.program,
            (matrices, sprite_info),
            [texture, normal_map],
            draw_unit,
            &DrawParams {
                depth_test: Some(DepthTest::default()),
                ..DrawParams::default()
            },
        );

        Ok(())
    }
}
