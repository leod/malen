use thiserror::Error;

use golem::GolemError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("no window")]
    NoWindow,

    #[error("no document")]
    NoDocument,

    #[error("could not find HTML element with id `{0}`")]
    InvalidElementId(String),

    #[error("HTML element with id `{0}` is not a canvas")]
    ElementIsNotCanvas(String),

    #[error("error from golem crate: {0}")]
    Golem(GolemError),

    #[error("could not get WebGL1 context")]
    GetContext(String),

    #[error("could not initialize WebGL1")]
    InitializeWebGl,

    #[error("Failed to load font")]
    Font(String),
}