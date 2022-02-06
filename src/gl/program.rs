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
    pub defines: Vec<(String, String)>,
    pub includes: Vec<String>,
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

fn shader_sources<U, V, const N: usize, const S: usize, const A: usize>(
    def: &ProgramDef<N, S, A>,
) -> [(u32, String); 2]
where
    U: UniformDecls,
    V: VertexDecls,
{
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

    let remove_outer_braces = |s: &str| {
        if s.starts_with("{") {
            s[1..s.len() - 1].to_string()
        } else {
            s.to_string()
        }
    };
    let preproc = |s: &str| {
        let code = def
            .includes
            .iter()
            .map(|c| remove_outer_braces(c))
            .collect::<Vec<_>>()
            .join("\n")
            + &remove_outer_braces(s);
        def.defines.iter().fold(code, |code, (from, to)| {
            code.replace(&format!("{{ {{ {0} }} }}", from), to)
                .replace(&format!("{{{{{0}}}}}", from), to)
        })
    };

    [
        (
            glow::VERTEX_SHADER,
            header.clone()
                + &vertex_source_header(&V::attributes(&attributes))
                + &preproc(&def.vertex_source),
        ),
        (
            glow::FRAGMENT_SHADER,
            header + &preproc(&def.fragment_source),
        ),
    ]
}

fn create_program<U, V, const N: usize, const S: usize, const A: usize>(
    gl: &Context,
    def: &ProgramDef<N, S, A>,
) -> Result<glow::Program, Error>
where
    U: UniformDecls,
    V: VertexDecls,
{
    let sources = shader_sources::<U, V, N, S, A>(def);
    let attributes = def
        .attributes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let program = unsafe { gl.create_program().map_err(Error::Glow)? };

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

pub type Glsl = &'static str;

#[macro_export]
macro_rules! glsl {
    { $code:tt } => { stringify!($code) };
    { $head:tt $($tail:tt)+ } => { $crate::glsl!{{$head $($tail)*}} }
}

#[macro_export]
macro_rules! program {
    {
        program $name:ident
        $(params { $($param_name:ident : $param_type:ty),* $(,)? })?
        $(uniforms { $($uniform_name:ident : $uniform_type:ty = $uniform_binding:expr),* $(,)? } )?
        $(samplers { $($sampler_name:ident : Sampler2),* $(,)? })?
        $(attributes { $($attribute_name:ident : $attribute_type:ty),* $(,)? })?
        $(defines [ $($define_name:ident => $define_value:expr),* $(,)? ])?
        $(includes [ $($include:expr),* $(,)? ])?
        vertex glsl!$vertex_source:tt
        fragment glsl!$fragment_source:tt
    } => {
        pub struct $name(
            pub <$name as std::ops::Deref>::Target,
        );

        impl $name {
            pub fn def(
                $($($param_name : $param_type),*)?
            ) -> $crate::gl::ProgramDef<
                { let x: &[&str] = &[$($(stringify!($uniform_name)),*)?]; x.len() },
                { let x: &[&str] = &[$($(stringify!($sampler_name)),*)?]; x.len() },
                { let x: &[&str] = &[$($(stringify!($attribute_name)),*)?]; x.len() },
            > {
                let define_names: Vec<String> = vec![$($(stringify!($define_name).into()),*)?];
                let define_values: Vec<String> = vec![$($($define_value.to_string()),*)?];

                $crate::gl::ProgramDef {
                    uniforms: [
                        $($((stringify!($uniform_name).into(), $uniform_binding)),*)?
                    ],
                    samplers: [
                        $($(stringify!($sampler_name).into()),*)?
                    ],
                    attributes: [
                        $($(stringify!($attribute_name).into()),*)?
                    ],
                    includes: vec![$($($include.to_string()),*)?],
                    defines: define_names.into_iter().zip(define_values.into_iter()).collect(),
                    vertex_source: stringify!($vertex_source).into(),
                    fragment_source: stringify!($fragment_source).into(),
                }
            }

            pub fn new(
                gl: Rc<gl::Context>,
                $($($param_name : $param_type),*)?
            ) -> Result<Self, gl::Error> {
                let program_def = Self::def(
                    $($($param_name),*)?
                );
                let program = $crate::gl::Program::new(gl, program_def)?;
                Ok($name(program))
            }
        }

        impl std::ops::Deref for $name {
            #[allow(unused_parens)]
            type Target = $crate::gl::Program<
                ($($($uniform_type),*)?),
                ($($($attribute_type),*)?),
                { let x: &[&str] = &[$($(stringify!($sampler_name)),*)?]; x.len() },
            >;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}
