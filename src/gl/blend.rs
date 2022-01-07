use crate::Color4;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Blend {
    pub equation: BlendEquation,
    pub func: BlendFunc,
    pub constant_color: Color4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlendEquation {
    pub color: BlendOp,
    pub alpha: BlendOp,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlendFunc {
    pub src_color: BlendFactor,
    pub src_alpha: BlendFactor,
    pub dst_color: BlendFactor,
    pub dst_alpha: BlendFactor,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturate,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

impl Default for Blend {
    fn default() -> Self {
        Self {
            equation: BlendEquation::default(),
            func: BlendFunc::default(),
            constant_color: Color4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl Default for BlendEquation {
    fn default() -> BlendEquation {
        Self::same(BlendOp::Add)
    }
}

impl Default for BlendFunc {
    fn default() -> Self {
        Self::same(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha)
    }
}

impl BlendEquation {
    pub fn same(op: BlendOp) -> Self {
        Self {
            color: op,
            alpha: op,
        }
    }

    pub fn is_same(self) -> bool {
        self.color == self.alpha
    }
}

impl BlendOp {
    pub fn to_gl(self) -> u32 {
        use BlendOp::*;

        match self {
            Add => glow::FUNC_ADD,
            Subtract => glow::FUNC_SUBTRACT,
            ReverseSubtract => glow::FUNC_REVERSE_SUBTRACT,
            Max => glow::MAX,
            Min => glow::MIN,
        }
    }
}

impl BlendFunc {
    pub fn same(src: BlendFactor, dst: BlendFactor) -> Self {
        Self {
            src_color: src,
            src_alpha: src,
            dst_color: dst,
            dst_alpha: dst,
        }
    }

    pub fn is_same(self) -> bool {
        self.src_color == self.src_alpha && self.dst_color == self.dst_alpha
    }
}

impl BlendFactor {
    pub fn to_gl(self) -> u32 {
        use BlendFactor::*;

        match self {
            Zero => glow::ZERO,
            One => glow::ONE,
            SrcColor => glow::SRC_COLOR,
            OneMinusSrcColor => glow::ONE_MINUS_SRC_COLOR,
            DstColor => glow::DST_COLOR,
            OneMinusDstColor => glow::ONE_MINUS_DST_COLOR,
            SrcAlpha => glow::SRC_ALPHA,
            OneMinusSrcAlpha => glow::ONE_MINUS_SRC_ALPHA,
            DstAlpha => glow::DST_ALPHA,
            OneMinusDstAlpha => glow::ONE_MINUS_DST_ALPHA,
            SrcAlphaSaturate => glow::SRC_ALPHA_SATURATE,
            ConstantColor => glow::CONSTANT_COLOR,
            OneMinusConstantColor => glow::ONE_MINUS_CONSTANT_COLOR,
            ConstantAlpha => glow::CONSTANT_ALPHA,
            OneMinusConstantAlpha => glow::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}
