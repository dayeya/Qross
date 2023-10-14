/*
image compression using QOI, official website: https://qoiformat.org/, you can find the specification there. 
*/

use crate::consts::*;
use crate::qoi_file::QoiFile;
use crate::pixel::{Pixel, Zero};
use crate::qoi_errror::QoiError;

extern crate rayon;
use itertools::Itertools;
use rayon::prelude::*;

use image::DynamicImage;
use std::fs::{File, self};
use std::path::{Path, PathBuf};
use std::io::{BufWriter, Write, BufReader, Read, Seek, Error};

// Read N bytes from 'reader'.
fn read_from_buffer<const N: usize>(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; N], QoiError> { 
    let mut bytes: [u8; N] = [u8::MIN; N];
    reader.read_exact(&mut bytes)?;

    *read_bytes += N;
    Ok(bytes)
}

// Modified read_from_buffer to read one bytes.
fn read_u8(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; 1], QoiError> {
    read_from_buffer::<1>(reader, read_bytes)
}

// Modified read_from_buffer to read 4 bytes.
fn read_u32(reader: &mut BufReader<File>, read_bytes: &mut usize) -> Result<[u8; 4], QoiError> {
    read_from_buffer::<4>(reader, read_bytes)
}

pub struct Data {
    pub path: String, 
    pub img: DynamicImage
}

pub struct Package {
    pub collection: Vec<Data>
}

impl Package {
    
    /*
    Returns a Package built from 'files'. 
     */
    pub fn with_files(files: Vec<String>) -> Self {
        Self { collection: 
            files
            .iter()
            .map(|p: &String| { 
                Data {
                    path: p.to_string(),
                    img: image::open(Path::new(p)).unwrap(),
                }
            })
            .collect_vec()      
        }
    }

    // Compresses all files in Package.
    pub fn compress_all(&mut self) {
        self.collection.par_iter_mut().for_each(|d| { let _ = d.compress(); });
    }
}

pub trait QoiEncode {
    fn encode(&self, pixels: &mut Vec<u8>, buffer: &mut BufWriter<File>) -> Result<usize, Error>;
}

pub trait QoiDecode { 
    fn decode(&self, buffer: &mut BufReader<File>, path: PathBuf) -> Result<QoiFile, QoiError>;
}

impl Data {

    pub fn get_pixels(&self) -> Vec<u8> {
        self.img.to_rgb8().into_raw()
    }

    pub fn compress(&self) -> Result<(), QoiError> {

        let mut img_name = Path::new(&self.path)
                .file_name()
                .unwrap_or_else(|| panic!("Problem at reading file!")).to_str().unwrap();
        img_name = &img_name[0..img_name.len() - 4];
        
        let encoded_suffix = img_name.to_owned() + "_encoded.qoi";
        let decoded_suffix = img_name.to_owned() + "_decoded.qoi";

        let encoded_path = Path::new(IMG_FOLDER_PATH).join(encoded_suffix);
        let decoded_path = Path::new(IMG_FOLDER_PATH).join(decoded_suffix);
 
        let mut buf_writer: BufWriter<File> = BufWriter::new(File::create(&encoded_path).unwrap());
        let bytes: usize = self.encode(&mut self.get_pixels(), &mut buf_writer)?; // encoded bytes. 
        let encoded_size: usize = fs::metadata(&encoded_path)?.len() as usize;

        assert_eq!(bytes, encoded_size);

        let mut buf_reader: BufReader<File> = BufReader::new(File::open(&encoded_path).unwrap());
        let mut qoi_file: QoiFile = self.decode(&mut buf_reader, decoded_path)
        .unwrap_or_else(|e| panic!("{}", e));

        // parse the pixels to the QOI image.
        qoi_file.set_size(); 
        qoi_file.create(qoi_file.clone().path);

        Ok(())
    }
}

impl QoiEncode for Data { 

    // QOI encoding function.
    fn encode(&self, pixels: &mut Vec<u8>, buffer: &mut BufWriter<File>) -> Result<usize, Error> {

        let mut written_bytes: usize = 0;
        let width: u32 = self.img.width();
        let height: u32 = self.img.height();

        let mut run: u8 = 0;
        let last_offset: usize = pixels.len() - CHANNELS as usize;

        let mut prev: Pixel = Pixel::zero();
        let mut seen_pixels: [Pixel; 64] = [Pixel::zero(); 64];

        #[allow(unused_assignments)]
        let mut index: usize = usize::MIN;

        let mut write = |chunk: &[u8]| {
            written_bytes += chunk.len();
            buffer.write_all(chunk)
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
        write(&QOI_MAGIC)?;
        write(&width.to_be_bytes())?;
        write(&height.to_be_bytes())?;
        write(&[CHANNELS])?;
        write(&[COLORSPACE])?;

        for offset in (0..pixels.len()).into_iter().step_by(CHANNELS as usize) {

            let pixel: Pixel = offset_pixel(offset);
            
            // check run.
            if pixel == prev { 
                run += 1;
                if let true = (run == 62 || offset == last_offset) { 
                    write(&[QOI_OP_RUN | (run - 1)])?; 
                    run = 0;
                }
            }

            // run existing and the pixel broke the equality.
            else {

                if run > 0 {
                    write(&[QOI_OP_RUN | (run - 1)])?; 
                    run = 0;
                }

                // check for index chunk.
                index = pixel.hash() % seen_pixels.len();
                if pixel == seen_pixels[index] {
                    write(&[QOI_OP_INDEX | index as u8])?;
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

                    if diff_r > -3 && diff_r < 2 
                    && diff_g > -3 && diff_g < 2
                    && diff_b > -3 && diff_b < 2 {

                        let qoi_diff_chunk: u8 = QOI_OP_DIFF as u8
                                | ((diff_r + 2) << 4) as u8
                                | ((diff_g + 2) << 2) as u8
                                | ((diff_b + 2) << 0) as u8; // clearer vision of the DIFF chunk.


                        write(&[qoi_diff_chunk])?;
                    }  
                    else {
                        
                        if  diff_g > -33 && diff_g < 32 
                            && dr_dg > -9 && dr_dg < 8
                            && db_dg > -9 && db_dg < 8 {
                            let qoi_luma_h: u8 = QOI_OP_LUMA | (diff_g + 32) as u8;
                            let qoi_luma_l: u8 = ((dr_dg + 8) << 4) as u8
                                               | ((db_dg + 8) << 0) as u8 ; // clearer vision of the LUMA chunk.

                            write(&[qoi_luma_h])?;
                            write(&[qoi_luma_l])?;
                        }
                        else {
                            // write 4 bytes of QOI_OP_RGB 
                            write(&[QOI_OP_RGB])?;
                            write(&[pixel.r])?;
                            write(&[pixel.g])?;
                            write(&[pixel.b])?;
                        }
                    }
                }
            }

            prev = pixel.clone();
        }
        write(&QOI_END_MARK)?;

        // return the number of encoded bytes.
        buffer.flush()?;
        Ok(written_bytes)

    }
}

impl QoiDecode for Data {

    // QOI encoding function.
    fn decode(&self, reader: &mut BufReader<File>, path: PathBuf) -> Result<QoiFile, QoiError> {

        let width: u32 = self.img.width();
        let height: u32 = self.img.height(); 

        let mut read_bytes: usize = 0;
        let mut prev = Pixel::zero();
        let mut seen_pixels = [Pixel::zero(); 64];

        let mut read_pixels: Vec<Pixel> = Vec::with_capacity((width * height) as usize);
        let mut buf: Vec<u8> = Vec::new();

        reader.read_to_end(&mut buf)?;
        reader.rewind()?;

        let buffered_magic: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_width: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_height: [u8; 4] = read_u32(reader, &mut read_bytes)?;
        let buffered_channels: u8 = read_u8(reader, &mut read_bytes)?[0];
        let buffered_color_space: u8 = read_u8(reader, &mut read_bytes)?[0];

        if buffered_magic != QOI_MAGIC {
            let magic_error: String = format!("{:?}", buffered_magic);
            panic!("{}", QoiError::InvalidHeader(magic_error));
        }

        while read_bytes < buf.len() - QOI_END_MARK_SIZE {

            let current_byte: u8 = read_u8(reader, &mut read_bytes)?[0];

            // check single encoded pixel.
            if current_byte == QOI_OP_RGB {
                let r: u8 = read_u8(reader, &mut read_bytes)?[0];
                let g: u8 = read_u8(reader, &mut read_bytes)?[0];
                let b: u8 = read_u8(reader, &mut read_bytes)?[0];

                prev.r = r;
                prev.g = g;
                prev.b = b;
 
                let index = prev.hash() % seen_pixels.len();
                seen_pixels[index] = prev;
                read_pixels.push(prev);
 
                continue;
            }

            // check run.
            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_RUN { 
                let mut run_value: u8 = (current_byte & QOI_RUN_LENGTH_MASK) + 1;

                // check if QOI buffer starts with a run.
                if read_bytes == QOI_HEADER_SIZE + 1 as usize {
                    let index: usize = Pixel::zero().hash() % seen_pixels.len();
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

                let diff_r: u8 = ((current_byte & QOI_RED_DIFF)   >> 4).wrapping_sub(2);
                let diff_g: u8 = ((current_byte & QOI_GREEN_DIFF) >> 2).wrapping_sub(2);
                let diff_b: u8 = ((current_byte & QOI_BLUE_DIFF)  >> 0).wrapping_sub(2);
                

                // prev pixel whom diff was calculated.
                let pixel: Pixel = Pixel {
                    r: (diff_r.wrapping_add(prev.r)), 
                    g: (diff_g.wrapping_add(prev.g)), 
                    b: (diff_b.wrapping_add(prev.b)),
                    a: 255, 
                };

                read_pixels.push(pixel);

                let index = pixel.hash() % seen_pixels.len();
                seen_pixels[index] = pixel;
                prev = pixel;

                continue;
            } 
            if (current_byte & QOI_2BIT_TAG_MASK) == QOI_OP_LUMA {
                
                let diff_g: u8 = (current_byte & QOI_LUMA_DG).wrapping_sub(32);
                let next_byte: u8 = read_u8(reader, &mut read_bytes)?[0];

                let dr_dg: u8 = ((next_byte & QOI_LUMA_DRDG_MASK) >> 4).wrapping_sub(8); // higher half
                let db_dg: u8 = ((next_byte & QOI_LUMA_DBDG_MASK) >> 0).wrapping_sub(8); // lower half

                let diff_r: u8 = dr_dg.wrapping_add(diff_g);
                let diff_b: u8 = db_dg.wrapping_add(diff_g); 

                // prev pixel whom luma was calculated.
                let pixel: Pixel = Pixel {
                    r: (diff_r.wrapping_add(prev.r)), 
                    g: (diff_g.wrapping_add(prev.g)), 
                    b: (diff_b.wrapping_add(prev.b)),
                    a: 255, 
                };

                read_pixels.push(pixel);

                let index = pixel.hash() % seen_pixels.len();
                seen_pixels[index ] = pixel;
                prev = pixel;

                continue;
            }
        }

        let buffered_end_mark: [u8; 8] = read_from_buffer::<8>(reader, &mut read_bytes)?;
        if buffered_end_mark != QOI_END_MARK { 
            let end_mark_alert: String = format!("{:?}", buffered_end_mark);
            panic!("{}", QoiError::InvalidEndMark(end_mark_alert));
        }

        Ok(
            // size will be set later uppoin create.
            QoiFile {
                path,
                size: 0,
                width: u32::from_be_bytes(buffered_width), 
                height: u32::from_be_bytes(buffered_height),
                channels: buffered_channels,
                color_space: buffered_color_space,
                pixels: read_pixels
            }
        )
    }
}