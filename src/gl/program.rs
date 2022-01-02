use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{Attribute, Context, Error, UniformBlocks, Vertex};

pub struct Program<U, V, const S: usize> {
    gl: Rc<Context>,
    program: glow::Program,
    _phantom: PhantomData<(U, V)>,
}

impl<U, V, const S: usize> Program<U, V, S> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }
}

pub struct ProgramDef<'a, const S: usize> {
    pub samplers: [&'a str; S],
    pub vertex_source: &'a str,
    pub fragment_source: &'a str,
}

impl<U, V, const S: usize> Program<U, V, S>
where
    U: UniformBlocks,
    V: Vertex,
{
    pub fn new(gl: Rc<Context>, def: ProgramDef<S>) -> Result<Self, Error> {
        let program = create_program::<U, S>(&*gl, &V::attributes(), def)?;

        Ok(Self {
            gl,
            program,
            _phantom: PhantomData,
        })
    }
}

fn create_program<U: UniformBlocks, const S: usize>(
    gl: &Context,
    attributes: &[Attribute],
    def: ProgramDef<S>,
) -> Result<<glow::Context as HasContext>::Program, Error> {
    let program = unsafe { gl.create_program().map_err(Error::Glow)? };

    let sampler_definitions = def
        .samplers
        .map(|sampler| format!("uniform sampler2D {};", sampler))
        .join("\n");

    let sources = [
        (
            glow::VERTEX_SHADER,
            SOURCE_HEADER.to_owned()
                + &vertex_source_header(attributes)
                + &U::glsl_definitions()
                + &sampler_definitions
                + def.vertex_source,
        ),
        (
            glow::FRAGMENT_SHADER,
            SOURCE_HEADER.to_owned()
                + &U::glsl_definitions()
                + &sampler_definitions
                + def.fragment_source,
        ),
    ];

    // TODO:
    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#compile_shaders_and_link_programs_in_parallel
    // https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices#dont_check_shader_compile_status_unless_linking_fails

    let shaders = sources
        .iter()
        .map(|(shader_type, shader_source)| {
            let shader = unsafe { gl.create_shader(*shader_type) }.map_err(Error::Glow)?;

            unsafe {
                gl.shader_source(shader, shader_source);
                gl.compile_shader(shader);

                log::info!("{}", shader_source);

                if !gl.get_shader_compile_status(shader) {
                    return Err(Error::OpenGL(format!(
                        "Shader failed to compile: {}",
                        gl.get_shader_info_log(shader)
                    )));
                }

                gl.attach_shader(program, shader);
            }

            Ok(shader)
        })
        .collect::<Result<Vec<_>, Error>>()?;

    // Binding attributes must be done before linking.
    for (index, attribute) in attributes.iter().enumerate() {
        unsafe {
            gl.bind_attrib_location(program, index as u32, attribute.name);
        }
    }

    unsafe {
        gl.link_program(program);

        if !gl.get_program_link_status(program) {
            return Err(Error::OpenGL(format!(
                "Program failed to link: {}",
                gl.get_program_info_log(program)
            )));
        }
    }

    // Once the program has been linked, the shader objects are no longer
    // required.
    for shader in shaders {
        unsafe {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
    }

    // Set texture uniforms.
    unsafe {
        gl.use_program(Some(program));
    }
    for (i, sampler) in def.samplers.iter().enumerate() {
        if let Some(location) = unsafe { gl.get_uniform_location(program, sampler) } {
            unsafe {
                gl.uniform_1_i32(Some(&location), i as i32);
            }
        } else {
            log::info!("Sampler `{}` (offset {}) is unused", sampler, i);
        }
    }

    // Setting uniform block binding locations should be done after linking.
    U::bind_to_program(gl, program);

    Ok(program)
}

const SOURCE_HEADER: &str = "#version 300 es\nprecision highp float;\n";

fn vertex_source_header(attributes: &[Attribute]) -> String {
    attributes
        .iter()
        .map(Attribute::glsl_string)
        .collect::<Vec<_>>()
        .join("")
        + "\n"
}

impl<U, V, const S: usize> Drop for Program<U, V, S> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}
