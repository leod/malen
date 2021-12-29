use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{Attribute, Context, Error, Vertex};

pub struct Program<V> {
    gl: Rc<Context>,
    program: <glow::Context as HasContext>::Program,
    _phantom: PhantomData<V>,
}

impl<V> Program<V> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }
}

impl<V: Vertex> Program<V> {
    pub fn new(gl: Rc<Context>, vertex_source: &str, fragment_source: &str) -> Result<Self, Error> {
        let program = create_program(gl, &V::attributes(), vertex_source, fragment_source)?;

        Ok(Self {
            gl,
            program,
            _phantom: PhantomData,
        })
    }
}

fn create_program(
    gl: Rc<Context>,
    attributes: &[Attribute],
    vertex_source: &str,
    fragment_source: &str,
) -> Result<<glow::Context as HasContext>::Program, Error> {
    let program = unsafe { gl.create_program().map_err(Error::Glow)? };

    let sources = [
        (
            glow::VERTEX_SHADER,
            SOURCE_HEADER.to_owned() + &vertex_source_header(attributes) + vertex_source,
        ),
        (
            glow::FRAGMENT_SHADER,
            SOURCE_HEADER.to_owned() + fragment_source,
        ),
    ];

    let shaders = sources
        .iter()
        .map(|&(shader_type, shader_source)| {
            let shader = unsafe { gl.create_shader(shader_type) }.map_err(Error::Glow)?;

            unsafe {
                gl.shader_source(shader, &shader_source);
                gl.compile_shader(shader);

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

    for (index, attribute) in attributes.into_iter().enumerate() {
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

    Ok(program)
}

const SOURCE_HEADER: &'static str = "#version 300 es\n";

fn vertex_source_header(attributes: &[Attribute]) -> String {
    attributes
        .iter()
        .map(Attribute::glsl_string)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

impl<V> Drop for Program<V> {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}
