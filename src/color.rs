use bytemuck_derive::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Default, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Color3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color3 {
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn to_color4(self) -> Color4 {
        Color4::new(self.r, self.g, self.b, 1.0)
    }

    pub fn to_linear(self) -> Color3 {
        Color3::new(self.r.powf(2.2), self.g.powf(2.2), self.b.powf(2.2))
    }
}

impl Color4 {
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_linear(self) -> Color4 {
        Color4::new(self.r.powf(2.2), self.g.powf(2.2), self.b.powf(2.2), self.a)
    }
}

impl From<[f32; 3]> for Color3 {
    fn from(c: [f32; 3]) -> Self {
        Self::new(c[0], c[1], c[2])
    }
}

impl From<Color3> for [f32; 3] {
    fn from(c: Color3) -> [f32; 3] {
        [c.r, c.g, c.b]
    }
}

impl From<[f32; 4]> for Color4 {
    fn from(c: [f32; 4]) -> Self {
        Self::new(c[0], c[1], c[2], c[3])
    }
}

impl From<Color4> for [f32; 4] {
    fn from(c: Color4) -> [f32; 4] {
        [c.r, c.g, c.b, c.a]
    }
}
