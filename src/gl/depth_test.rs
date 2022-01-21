#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DepthTest {
    pub func: DepthFunc,
    pub range_near: f32,
    pub range_far: f32,
    pub write: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DepthFunc {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl Default for DepthTest {
    fn default() -> Self {
        Self {
            func: DepthFunc::default(),
            range_near: 0.0,
            range_far: 1.0,
            write: true,
        }
    }
}

impl DepthTest {
    pub fn read_only() -> Self {
        Self {
            write: false,
            ..DepthTest::default()
        }
    }
}

impl Default for DepthFunc {
    fn default() -> Self {
        DepthFunc::Less
    }
}

impl DepthFunc {
    pub fn to_gl(self) -> u32 {
        use DepthFunc::*;

        match self {
            Never => glow::NEVER,
            Less => glow::LESS,
            Equal => glow::EQUAL,
            LessOrEqual => glow::LEQUAL,
            Greater => glow::GREATER,
            NotEqual => glow::NOTEQUAL,
            GreaterOrEqual => glow::GEQUAL,
            Always => glow::ALWAYS,
        }
    }
}
