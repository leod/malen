use std::{cell::RefCell, rc::Rc};

use crate::{
    data::SpriteVertex,
    gl::{self, DepthTest, DrawParams, DrawUnit, Element, Program, ProgramDef, Texture, Uniform},
    pass::{
        MatricesBlock, SpriteInfoBlock, SpriteInfos, MATRICES_BLOCK_BINDING,
        SPRITE_INFO_BLOCK_BINDING,
    },
};

use super::{super::ObjectLightParams, OBJECT_LIGHT_PARAMS_BLOCK_BINDING};

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
layout (location = 2) out vec4 f_occlusion;

void main() {
    f_albedo = vec4(pow(texture(sprite, v_uv).rgb, vec3(2.2)), object_light_params.ambient_scale);
    f_normal = texture(normal_map, v_uv);
    f_occlusion = vec4(object_light_params.occlusion, 0.0, 0.0, 1.0);
}
"#;

pub struct GeometrySpriteWithNormalsPass {
    program: Program<(MatricesBlock, SpriteInfoBlock, ObjectLightParams), SpriteVertex, 2>,
    sprite_infos: RefCell<SpriteInfos>,
}

impl GeometrySpriteWithNormalsPass {
    pub fn new(gl: Rc<gl::Context>) -> Result<Self, gl::Error> {
        let program_def = ProgramDef {
            uniform_blocks: [
                ("matrices", MATRICES_BLOCK_BINDING),
                ("sprite_info", SPRITE_INFO_BLOCK_BINDING),
                ("object_light_params", OBJECT_LIGHT_PARAMS_BLOCK_BINDING),
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
        matrices: &Uniform<MatricesBlock>,
        object_light_params: &Uniform<ObjectLightParams>,
        texture: &Texture,
        normal_map: &Texture,
        draw_unit: DrawUnit<SpriteVertex, E>,
    ) -> Result<(), gl::Error>
    where
        E: Element,
    {
        let mut sprite_infos = self.sprite_infos.borrow_mut();
        let sprite_info = sprite_infos.get(texture)?;

        gl::draw(
            &self.program,
            (matrices, sprite_info, object_light_params),
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
