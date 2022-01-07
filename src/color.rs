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

    pub fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[self.r, self.g, self.b])
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

    pub fn write(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&[self.r, self.g, self.b, self.a])
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
