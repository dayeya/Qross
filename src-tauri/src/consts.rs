
// Path to all saved files.
pub const IMG_FOLDER_PATH: &str = r"C:\coding\RUST\image_compressor\src\app_imgs\";

// Important QOI file fields.
pub const QOI_MAGIC: [u8; 4] = [b'q', b'o', b'i', b'f'];
pub const QOI_END_MARK: [u8; 8] = [0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b0, 0b1];

// One-byte header fields.
pub const CHANNELS: u8 = 3;
pub const COLORSPACE: u8 = 1;

pub const QOI_HEADER_SIZE: usize = [u8::MIN; 14].len();
pub const QOI_END_MARK_SIZE: usize = [u8::MIN; 8].len();

// Tags
pub const QOI_OP_RGB:   u8 = 0b11111110;
pub const QOI_OP_INDEX: u8 = 0b00000000;
pub const QOI_OP_DIFF:  u8 = 0b01000000;
pub const QOI_OP_RUN:   u8 = 0b11000000;
pub const QOI_OP_LUMA:  u8 = 0b10000000;

// Masks
pub const QOI_RGB_MASK:         u8 = 0b11111111;
pub const QOI_2BIT_TAG_MASK:    u8 = 0b11000000;
pub const QOI_RUN_LENGTH_MASK:  u8 = 0b00111111;
pub const QOI_INDEX_VALUE_MASK: u8 = 0b00111111; 

// Luma masks
pub const QOI_LUMA_DG:        u8 = 0b00111111;
pub const QOI_LUMA_DRDG_MASK: u8 = 0b11110000;
pub const QOI_LUMA_DBDG_MASK: u8 = 0b00001111;

pub const QOI_RED_DIFF:   u8 = 0b00110000;
pub const QOI_GREEN_DIFF: u8 = 0b00001100;
pub const QOI_BLUE_DIFF:  u8 = 0b00000011;

