/*
image compression using QOI, official website: https://qoiformat.org/, you can find the specification there. 
*/

#[allow(dead_code, unused_variables)]
use crate::consts::*;
use crate::qoi_errror::QoiError;
use crate::pixel::{Pixel, Zero};

use std::fs::File;
use std::io::{BufWriter, Write, BufReader, Read}; 
use image::{DynamicImage, ImageBuffer, Rgb, ImageFormat};

fn read_from_buffer<const N: usize>(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; N], QoiError> { 
    let mut bytes: [u8; N] = [u8::MIN; N];
    reader.read_exact(&mut bytes)?;

    *read_bytes += bytes.len();
    println!("{:?} at byte: {:X}", bytes, read_bytes);

    Ok(bytes)
}

fn read_u8(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; 1], QoiError> {
    read_from_buffer::<1>(reader, read_bytes)
}

fn read_u32(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; 4], QoiError> {
    read_from_buffer::<4>(reader, read_bytes)
}

pub struct Data {
    pub path: String, 
    pub img: DynamicImage
}

#[derive(Clone)]
pub struct QoiFile {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub color_space: u8,
    pub pixels: Vec<Pixel>,
}

impl QoiFile {

    fn parse_pixels_to_vec(&self, px_buffer: &mut Vec<u8>) {
        for px in self.pixels.clone().into_iter() {
            let bytes: [u8; CHANNELS as usize] = px.to_bytes();
            px_buffer.push(bytes[CHANNELS as usize - 3]); //  first channel
            px_buffer.push(bytes[CHANNELS as usize - 2]); // second channel
            px_buffer.push(bytes[CHANNELS as usize - 1]); //  third channel
        }
    }

    fn create(&self, path: String) { 
        let mut px_buffer: Vec<u8> = Vec::with_capacity((self.width * self.height * self.channels as u32) as usize);

        // insert all pixels as bytes into the pixel buffer.
        self.parse_pixels_to_vec(&mut px_buffer);

        let converted: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(self.width, self.height, px_buffer).unwrap();
        let _ = converted.save_with_format(path, ImageFormat::Qoi)
        .expect(&format!("{}", 
        QoiError::SavingError(format!("Failed at saving file"))
        ));
    }
}

pub trait QoiEncode {
    fn encode(&self, pixels: &mut Vec<u8>, buffer: &mut BufWriter<File>) -> Option<usize>;
}

pub trait QoiDecode {
    fn decode(&self, buffer: &mut BufReader<File>, path: String) -> Result<QoiFile, QoiError>;
}

impl Data {

    pub fn get_pixels(&self) -> Vec<u8> {
        self.img.to_rgb8().into_raw()
    }

    pub fn compress(&self, qoi_file_path: String) {

        let mut buf_writer: BufWriter<File> = BufWriter::new(File::create(&qoi_file_path).unwrap());
        let bytes: Option<usize> = self.encode(&mut self.get_pixels(), &mut buf_writer);    
        println!("{} bytes were encoded from {}", bytes.unwrap(), self.path);
    
        let mut buf_reader: BufReader<File> = BufReader::new(File::open(&qoi_file_path).unwrap());
        let qoi_file: Result<QoiFile, QoiError> = self.decode(&mut buf_reader, qoi_file_path);

        // parse the pixels to the QOI image.
        match qoi_file {
            Ok(file) => file.create(file.clone().path), 
            Err(e) => panic!("{}", e),
        };
    }
}

impl QoiEncode for Data { 

    // QOI encoding function.
    fn encode(&self, pixels: &mut Vec<u8>, buffer: &mut BufWriter<File>) -> Option<usize> {

        let mut written_bytes: usize = 0;
        let width: u32 = self.img.width();
        let height: u32 = self.img.height();

        let mut run: u8 = 0;
        let last_offset: usize = pixels.len() - CHANNELS as usize;

        let mut prev: Pixel = Pixel::zero();
        let mut seen_pixels: [Pixel; 64] = [Pixel::zero(); 64];

        #[allow(unused_assignments)]
        let mut index: usize = usize::MIN; 

        // const MAX_SIZE: u32 = width * height * (CHANNELS as u32 + 1) + QOI_HEADER_SIZE as u32 + QOI_END_MARK_SIZE as u32;
        // let bytes: [u8; MAX_SIZE as usize]; 

        let mut write = |chunk: &[u8]| {
            
            let msg = format!("error when writing to buffer -> {:?}", chunk); 
            let _ = buffer.write_all(chunk).expect(&msg);
            written_bytes += chunk.len();
        };

        let offset_pixel = |offset: usize| -> Pixel { 
            Pixel {
                r: pixels[offset],
                g: pixels[offset + 1],
                b: pixels[offset + 2],
                a: 255 as u8,
            }
        };

        // write the header into the buffer.
        write(&QOI_MAGIC.to_be_bytes());
        write(&width.to_be_bytes());
        write(&height.to_be_bytes());
        write(&[CHANNELS]);
        write(&[COLORSPACE]);

        for offset in (0..pixels.len()).into_iter().step_by(CHANNELS as usize) {

            let pixel: Pixel = offset_pixel(offset);
            
            // check run.
            if pixel == prev { 
                run += 1;
                if let true = (run == 62 || offset == last_offset) { 
                    write(&(QOI_OP_RUN | (run - 1)).to_be_bytes()); 
                    run = 0;
                }
            }

            // run existing and the pixel broke the equality.
            else {

                if run > 0 {
                    write(&(QOI_OP_RUN | (run - 1)).to_be_bytes()); 
                    run = 0
                }

                // check for index chunk.
                index = pixel.hash() % (seen_pixels.len() as usize);
                if pixel == offset_pixel(pixels[index] as usize) {
                    write(&(QOI_OP_INDEX | index as u8).to_be_bytes());
                }
                else {

                    // update the array.
                    seen_pixels[index] = pixel.clone(); 
  
                    // check for diff chunk.
                    let diff_r = pixel.r as i16 - prev.r as i16;
                    let diff_g = pixel.g as i16 - prev.g as i16;
                    let diff_b = pixel.b as i16 - prev.b as i16;

                    let dr_dg = diff_r - diff_g;
                    let db_dg = diff_b - diff_g;

                    if diff_r >= -2 && diff_r <= 1 
                    && diff_g >= -2 && diff_g <= 1
                    && diff_b >= -2 && diff_b <= 1 {

                        let qoi_diff_chunk: i16 = QOI_OP_DIFF as i16
                                | ((diff_r + 2) << 4)
                                | ((diff_g + 2) << 2)
                                | ((diff_b + 2) << 0); // clearer vision of the DIFF chunk.

                        write(&qoi_diff_chunk.to_be_bytes());
                    }  
                    else {
                        
                        if diff_g >= -32 && diff_g <= 31 
                        && dr_dg >= -8 && dr_dg <= 7
                        && db_dg >= -8 && db_dg <= 7 {
                            let qoi_luma_h: u8 = QOI_OP_LUMA | (diff_g + 32) as u8;
                            let qoi_luq_l: u8 = ((dr_dg + 8) << 4) as u8 | ((db_dg + 8) << 0) as u8 ; // clearer vision of the LUMA chunk.

                            write(&qoi_luma_h.to_be_bytes());
                            write(&qoi_luq_l.to_be_bytes());
                        }
                        else {
                            // write 4 bytes of QOI_OP_RGB 
                            write(&[QOI_OP_RGB]);
                            write(&[pixel.r]);
                            write(&[pixel.g]);
                            write(&[pixel.b]);
                        }
                    }
                }
            }
            prev = pixel;
        }
        write(&QOI_END_MARK.to_be_bytes());

        // return the number of encoded bytes.
        Some(written_bytes)

    }
}

impl QoiDecode for Data {

    // QOI encoding function.
    fn decode(&self, reader: &mut BufReader<File>, path: String) -> Result<QoiFile, QoiError> {
        
        // check empty buffer.
        assert!(reader.buffer().is_empty());

        let width: u32 = self.img.width();
        let height: u32 = self.img.height();

        let mut prev: Pixel = Pixel::zero();
        let mut seen_pixels: [Pixel; 64] = [Pixel::zero(); 64];
        let max_size: usize = QOI_HEADER_SIZE + width as usize * height as usize * CHANNELS as usize + QOI_END_MARK_SIZE; // bytes

        let mut read_bytes: usize = 0;
        let mut read_pixels: Vec<Pixel> = Vec::new();

        let buffered_magic: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_width: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_height: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_channels: u8 = read_u8(reader, &mut read_bytes)?[0];
        let buffered_color_space: u8 = read_u8(reader, &mut read_bytes)?[0];

        println!("{}", u32::from_ne_bytes(buffered_width));
        println!("{}", u32::from_ne_bytes(buffered_height));
        println!("{}", buffered_channels);
        println!("{}", buffered_color_space);

        if buffered_magic != QOI_MAGIC.to_be_bytes() {
            let magic_error: String = format!("{:?} -> u32: {}", buffered_magic, u32::from_ne_bytes(buffered_magic));
            panic!("{}", QoiError::InvalidHeader(magic_error));
        }

        while read_bytes <= max_size - QOI_END_MARK_SIZE {
            let current_byte: u8 = read_u8(reader, &mut read_bytes)?[0];

            // check single encoded pixel.
            if current_byte == QOI_OP_RGB {
                let r: u8 = read_u8(reader, &mut read_bytes)?[0];
                let b: u8 = read_u8(reader, &mut read_bytes)?[0];
                let g: u8 = read_u8(reader, &mut read_bytes)?[0];

                prev.r = r;
                prev.g = g;
                prev.b = b;

                let index = prev.hash() % (seen_pixels.len() as usize);
                read_pixels.push(prev);
                seen_pixels[index] = prev;

                continue;
            }

            // check run.
            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_RUN { 
                let mut run_value: u8 = (current_byte & QOI_RUN_LENGTH_MASK) + 1;

                // check if QOI buffer starts with a run.
                if read_bytes == QOI_HEADER_SIZE + 1 as usize {
                    let index: usize = Pixel::zero().hash() % (seen_pixels.len() as usize);
                    seen_pixels[index] = Pixel::zero();
                }

                while run_value > 0 {
                    read_pixels.push(prev);
                    run_value -= 1;
                }

                continue;
            }

            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_INDEX {
                let index: u8 = current_byte & QOI_INDEX_VALUE_MASK;
                let px: Pixel = seen_pixels[index as usize];
                read_pixels.push(px);

                prev = px;

                continue;
            }

            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_DIFF {

                println!("{}", (current_byte & QOI_RED_DIFF) >> 4);

                let diff_r: u8 = ((current_byte & QOI_RED_DIFF)   >> 4) - 2;
                let diff_g: u8 = ((current_byte & QOI_GREEN_DIFF) >> 2) - 2;
                let diff_b: u8 = ((current_byte & QOI_BLUE_DIFF)  >> 0) - 2;
                

                // prev pixel whom diff was calculated.
                let pixel: Pixel = Pixel {
                    r: (diff_r + prev.r) & 0b11111111, 
                    g: (diff_g + prev.g) & 0b11111111, 
                    b: (diff_b + prev.b) & 0b11111111,
                    a: 255, 
                };

                read_pixels.push(pixel);

                let index = pixel.hash() % (seen_pixels.len() as usize);
                seen_pixels[index] = pixel;
                prev = pixel;

                continue;
            }

            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_LUMA {
                
                let diff_g: u8 = (current_byte & QOI_LUMA_DG) - 32;
                let next_byte: u8 = read_u8(reader, &mut read_bytes)?[0];

                let dr_dg: u8 = (next_byte & QOI_HALF_MASK) - 8; // higher half
                let db_dg: u8 = (next_byte & !QOI_HALF_MASK) - 8; // lower half

                let diff_r: u8 = dr_dg + diff_g;
                let diff_b: u8 = db_dg + diff_g; 

                // prev pixel whom luma was calculated.
                let pixel: Pixel = Pixel {
                    r: (diff_r + prev.r) & 0b11111111, 
                    g: (diff_g + prev.g) & 0b11111111, 
                    b: (diff_b + prev.b) & 0b11111111,
                    a: 255, 
                };

                read_pixels.push(pixel);

                let index = pixel.hash() % (seen_pixels.len() as usize);
                seen_pixels[index ] = pixel;
                prev = pixel;

                continue;
            }
        }

        println!("read bytes after loop {}", read_bytes);
        let buffered_end_mark: [u8; 8] = read_from_buffer::<8>(reader, &mut read_bytes)?;
        if buffered_end_mark != QOI_END_MARK.to_be_bytes() { 
            let end_mark_alert: String = format!("{:?}", buffered_end_mark);
            panic!("{}", QoiError::InvalidEndMark(end_mark_alert));
        }

        Ok(
            QoiFile {
                path: path,
                width: u32::from_ne_bytes(buffered_width), 
                height: u32::from_ne_bytes(buffered_height),
                channels: buffered_channels,
                color_space: buffered_color_space,
                pixels: read_pixels
            }
        )
    }
}

// SAVE FOR LATER read_to_end()