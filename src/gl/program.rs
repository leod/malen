use std::{marker::PhantomData, rc::Rc};

use glow::HasContext;

use super::{vertex::VertexDecls, Attribute, Context, Error, UniformDecls};

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

pub struct ProgramDef<const N: usize, const S: usize, const A: usize> {
    pub uniforms: [(String, u32); N],
    pub samplers: [String; S],
    pub attributes: [String; A],
    pub vertex_source: String,
    pub fragment_source: String,
}

impl<U, V, const S: usize> Program<U, V, S>
where
    U: UniformDecls,
    V: VertexDecls,
{
    pub fn new<const N: usize, const A: usize>(
        gl: Rc<Context>,
        def: ProgramDef<N, S, A>,
    ) -> Result<Self, Error> {
        // I think we'll be able to turn this into a compile time check once
        // there is a bit more const-generics on stable.
        assert!(N == U::N);
        assert!(A == V::N);

        let id = create_program::<U, V, N, S, A>(&*gl, &def)?;

        Ok(Self {
            gl,
            id,
            uniform_block_bindings: def.uniforms.iter().map(|(_, b)| *b).collect(),
            _phantom: PhantomData,
        })
    }
}

fn create_program<U, V, const N: usize, const S: usize, const A: usize>(
    gl: &Context,
    def: &ProgramDef<N, S, A>,
) -> Result<glow::Program, Error>
where
    U: UniformDecls,
    V: VertexDecls,
{
    let program = unsafe { gl.create_program().map_err(Error::Glow)? };

    let uniform_decls = U::glsl_decls(
        &def.uniforms
            .iter()
            .map(|(s, _)| s.as_str())
            .collect::<Vec<_>>(),
    );
    let attributes = def
        .attributes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let sampler_decls = def
        .samplers
        .iter()
        .map(|sampler| format!("uniform sampler2D {};", sampler))
        .collect::<Vec<_>>()
        .join("\n");

    let header = SOURCE_HEADER.to_owned() + &uniform_decls + &sampler_decls;

    let sources = [
        (
            glow::VERTEX_SHADER,
            header.clone()
                + &vertex_source_header(&V::attributes(&attributes))
                + &def.vertex_source,
        ),
        (glow::FRAGMENT_SHADER, header + &def.fragment_source),
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
    for (index, attribute) in V::attributes(&attributes).iter().enumerate() {
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
    U::bind_to_program(
        gl,
        program,
        &def.uniforms
            .iter()
            .map(|(s, b)| (s.as_str(), *b))
            .collect::<Vec<_>>(),
    );

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
macro_rules! program {
    {
        $name:ident [
            (
                $($uniform_name:ident : $uniform_type:ty = $uniform_binding:expr),* $(,)?
            ),
            (
                $($sampler_name:ident),* $(,)?
            ),
            (
                $($attribute_name:ident : $attribute_type:ty),* $(,)?
            ) $(,)?
        ]
        => (
            $vertex_source:expr,
            $fragment_source:expr $(,)?
        )
    } => {
        $crate::program! {
            | | $name [
                (
                    $($uniform_name : $uniform_type = $uniform_binding),*
                ),
                (
                    $($sampler_name),*
                ),
                (
                    $($attribute_name : $attribute_type),*
                ),
            ]
            => (
                $vertex_source,
                $fragment_source,
            )
        }
    };

    {
        |$($param_name:ident : $param_type:ty),* $(,)?|
        $name:ident [
            (
                $($uniform_name:ident : $uniform_type:ty = $uniform_binding:expr),* $(,)?
            ),
            (
                $($sampler_name:ident),* $(,)?
            ),
            (
                $($attribute_name:ident : $attribute_type:ty),* $(,)?
            ) $(,)?
        ]
        => (
            $vertex_source:expr,
            $fragment_source:expr $(,)?
        )
    } => {
        pub struct $name(
            pub <$name as std::ops::Deref>::Target,
        );

        impl $name {
            pub fn def(
                $($param_name : $param_type),*
            ) -> $crate::gl::ProgramDef<
                { let x: &[&str] = &[$(stringify!($uniform_name)),*]; x.len() },
                { let x: &[&str] = &[$(stringify!($sampler_name)),*]; x.len() },
                { let x: &[&str] = &[$(stringify!($attribute_name)),*]; x.len() },
            > {
                $crate::gl::ProgramDef {
                    uniforms: [
                        $((stringify!($uniform_name).into(), $uniform_binding)),*
                    ],
                    samplers: [
                        $(stringify!($sampler_name).into()),*
                    ],
                    attributes: [
                        $(stringify!($attribute_name).into()),*
                    ],
                    vertex_source: $vertex_source.into(),
                    fragment_source: $fragment_source.into(),
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
                { let x: &[&str] = &[$(stringify!($sampler_name)),*]; x.len() },
            >;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
