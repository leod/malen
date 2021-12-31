use thiserror::Error;

use crate::gl;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("no window")]
    NoWindow,

    #[error("no document")]
    NoDocument,

    #[error("could not find HTML element with id `{0}`")]
    InvalidElementId(String),

    #[error("HTML element with id `{0}` is not a canvas")]
    ElementIsNotCanvas(String),

    #[error("could not get WebGL1 context")]
    GetContext(String),

    #[error("could not initialize WebGL1")]
    InitializeWebGl,

    #[error("GL error: {0}")]
    OpenGL(#[from] gl::Error),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("GL error: {0}")]
    OpenGL(#[from] gl::Error),
}
