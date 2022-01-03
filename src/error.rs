use thiserror::Error;

use crate::{
    gl::{self, LoadTextureError, NewTextureError},
    text::LoadFontError,
    WriteTextError,
};

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

    #[error("OpenGL error: {0}")]
    OpenGL(#[from] gl::Error),

    #[error("new texture error: {0}")]
    NewTexture(#[from] NewTextureError),

    #[error("load texture error: {0}")]
    LoadTexture(#[from] LoadTextureError),

    #[error("load font error: {0}")]
    LoadFont(#[from] LoadFontError),
}

#[derive(Error, Debug)]
pub enum FrameError {
    #[error("OpenGL error: {0}")]
    OpenGL(#[from] gl::Error),

    #[error("write text error: {0}")]
    WriteTextError(#[from] WriteTextError),
}
