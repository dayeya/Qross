
// important QOI file fields. 
pub const QOI_MAGIC: u32 = u32::from_be_bytes([b'q', b'o', b'i', b'f']);
pub const QOI_END_MARK: u64 = u64::from_be_bytes([0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b1]);

// one byte header fields.
pub const CHANNELS: u8 = 3;
pub const COLORSPACE: u8 = 1;

pub const QOI_HEADER_SIZE: usize = [u8::MIN; 14].len();
pub const QOI_END_MARK_SIZE: usize = [u8::MIN; 8].len();

// tags
pub const QOI_OP_RGB: u8 = 0b11111110;
pub const QOI_OP_INDEX: u8 = 0b00;
pub const QOI_OP_DIFF: u8 = 0b01;
pub const QOI_OP_RUN: u8 = 0b11;
pub const QOI_OP_LUMA: u8 = 0b10;

// masks
pub const QOI_2BIT_TAG_MASK: u8 =  0b11000000;
pub const QOI_RGB_MASK: u8 = 0b11111111;
pub const QOI_RUN_LENGTH_MASK: u8 = !QOI_2BIT_TAG_MASK; // 0b00111111
pub const QOI_INDEX_VALUE_MASK: u8 = !QOI_2BIT_TAG_MASK; 

// luma masks
pub const QOI_LUMA_DG: u8 =   0b00111111;
pub const QOI_HALF_MASK: u8 = 0b11110000; 

pub const QOI_RED_DIFF: u8 =   0b00110000;
pub const QOI_GREEN_DIFF: u8 = 0b00001100;
pub const QOI_BLUE_DIFF: u8 =  0b00000011;

