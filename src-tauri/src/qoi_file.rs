use std::path::PathBuf;
use image::{ImageBuffer, ImageFormat, Rgb};

use crate::pixel::Pixel;
use crate::qoi_errror::QoiError;

#[derive(Clone)]
pub struct QoiFile {
    pub path: PathBuf,
    pub size: usize,
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub color_space: u8,
    pub pixels: Vec<Pixel>,
}

impl QoiFile {
    pub fn set_size(&mut self) { 
        self.size = (self.width * self.height * self.channels as u32) as usize;
    }
    
    pub fn parse_pixels_to_vec(&self, px_buffer: &mut Vec<u8>) {
        self.pixels.iter().for_each(
            |px| px_buffer.extend(px.to_bytes())
        )
    }
    
    pub fn create(&mut self, path: PathBuf) {
        let mut px_buffer: Vec<u8> = Vec::with_capacity(self.size);
        self.parse_pixels_to_vec(&mut px_buffer);

        let converted: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(self.width, self.height, px_buffer).unwrap();

        converted.save_with_format(path, ImageFormat::Qoi).expect(&format!("{}", 
            QoiError::SavingError(format!("Failed at saving file"))
        ));
    }
}
