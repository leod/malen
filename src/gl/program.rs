use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{vertex::VertexDecls, Attribute, Context, Error, UniformBlockDecls};

pub struct Program<U, V, const S: usize> {
    gl: Rc<Context>,
    id: glow::Program,
    uniform_block_bindings: Vec<u32>,
    _phantom: PhantomData<(U, V)>,
}

impl<U, V, const S: usize> Program<U, V, S> {
    pub fn gl(&self) -> Rc<Context> {
        self.gl.clone()
    }

    pub fn id(&self) -> glow::Program {
        self.id
    }

    pub fn uniform_block_bindings(&self) -> &[u32] {
        &self.uniform_block_bindings
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.id));
        }
    }
}

pub struct ProgramDef<'a, const N: usize, const S: usize> {
    pub uniforms: [(&'a str, u32); N],
    pub samplers: [&'a str; S],
    pub vertex_source: &'a str,
    pub fragment_source: &'a str,
}

impl<U, V, const S: usize> Program<U, V, S>
where
    U: UniformBlockDecls,
    V: VertexDecls,
{
    pub fn new<const N: usize>(gl: Rc<Context>, def: ProgramDef<N, S>) -> Result<Self, Error> {
        // I think we'll be able to turn this into a compile time check once
        // there is a bit more const-generics on stable.
        assert!(N == U::N);

        let id = create_program::<U, V, N, S>(&*gl, &def)?;

        Ok(Self {
            gl,
            id,
            uniform_block_bindings: def.uniform_block_bindings().to_vec(),
            _phantom: PhantomData,
        })
    }
}

impl<'a, const N: usize, const S: usize> ProgramDef<'a, N, S> {
    fn uniform_block_names(&self) -> [&str; N] {
        let mut result = [""; N];
        for (i, (instance_name, _)) in self.uniforms.iter().enumerate() {
            result[i] = instance_name;
        }
        result
    }

    fn uniform_block_bindings(&self) -> [u32; N] {
        let mut result = [0; N];
        for (i, (_, binding)) in self.uniforms.iter().enumerate() {
            result[i] = *binding;
        }
        result
    }
}

fn create_program<U, V, const N: usize, const S: usize>(
    gl: &Context,
    def: &ProgramDef<N, S>,
) -> Result<glow::Program, Error>
where
    U: UniformBlockDecls,
    V: VertexDecls,
{
    let program = unsafe { gl.create_program().map_err(Error::Glow)? };

    let sampler_definitions = def
        .samplers
        .map(|sampler| format!("uniform sampler2D {};", sampler))
        .join("\n");

    let sources = [
        (
            glow::VERTEX_SHADER,
            SOURCE_HEADER.to_owned()
                + &vertex_source_header(&V::attributes())
                + &U::glsl_definitions(&def.uniform_block_names())
                + &sampler_definitions
                + def.vertex_source,
        ),
        (
            glow::FRAGMENT_SHADER,
            SOURCE_HEADER.to_owned()
                + &U::glsl_definitions(&def.uniform_block_names())
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

                log::info!(
                    "{}",
                    shader_source
                        .split('\n')
                        .enumerate()
                        .map(|(i, line)| format!("{}: {}", i + 1, line))
                        .collect::<Vec<String>>()
                        .join("\n")
                );

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
    for (index, attribute) in V::attributes().iter().enumerate() {
        unsafe {
            gl.bind_attrib_location(program, index as u32, &attribute.name);
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
    U::bind_to_program(gl, program, &def.uniforms);

    Ok(program)
}

const SOURCE_HEADER: &str = r#"#version 300 es
    precision highp float;
    precision highp sampler2D;"#;

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
            self.gl.delete_program(self.id);
        }
    }
}

#[macro_export]
macro_rules! program_def {
    {
        name: $name:ident,
        params: (
            $($param_name:ident : $param_type:ty),* $(,)?
        ),
        uniforms: {
            $($uniform_name:ident : $uniform_type:ty = $uniform_binding:expr),* $(,)?
        },
        samplers: [
            $($sampler_name:literal),* $(,)?
        ],
        attributes: {
            $($attribute_name:ident : $attribute_type:ty),* $(,)?
        },
        vertex_source: $vertex_source:expr,
        fragment_source: $fragment_source:expr,
    } => {
        pub struct $name(
            pub <$name as std::ops::Deref>::Target,
        );

        impl $name {
            pub fn def(
                $($param_name : $param_type),*
            ) -> $crate::gl::ProgramDef<
                'static,
                { [$(stringify!($uniform_name)),*].len() },
                { [$($sampler_name),*].len() },
            > {
                $crate::gl::ProgramDef {
                    uniforms: [
                        $((stringify!($uniform_name), $uniform_binding)),*
                    ],
                    samplers: [
                        $($sampler_name),*
                    ],
                    vertex_source: $vertex_source,
                    fragment_source: $fragment_source,
                }
            }

            pub fn new(
                gl: Rc<gl::Context>,
                $($param_name : $param_type),*
            ) -> Result<Self, gl::Error> {
                let program_def = Self::def(
                    $($param_name),*
                );
                let program = $crate::gl::Program::new(gl, program_def)?;
                Ok($name(program))
            }
        }

        impl std::ops::Deref for $name {
            #[allow(unused_parens)]
            type Target = $crate::gl::Program<
                ($($uniform_type),*),
                ($($attribute_type),*),
                { [$($sampler_name),*].len() },
            >;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    }
}
