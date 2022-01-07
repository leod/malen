use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("OpenGL (glow) error: {0}")]
    Glow(String),

    #[error("OpenGL error: {0}")]
    OpenGL(String),
}
