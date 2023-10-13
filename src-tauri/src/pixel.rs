use crate::consts::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8, 
}

impl Pixel {
    pub fn to_bytes(&self) -> [u8; CHANNELS as usize] { 
        return [self.r, self.g, self.b];
    }
    pub fn hash(&self) -> usize {
        self.r as usize * 3 + self.g as usize * 5 + self.b as usize * 7
    }
}

pub trait Zero {
    fn zero() -> Pixel;
}

impl Zero for Pixel {
    fn zero() -> Pixel {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}
