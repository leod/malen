use thiserror::Error;

use golem::GolemError;

use crate::gl;

#[derive(Error, Debug)]
pub enum CanvasInitError {
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
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("error from golem crate: {0}")]
    Golem(GolemError),

    #[error("GL error: {0}")]
    Gl(gl::Error),

    #[error("Failed to load font")]
    Font(String),
}
