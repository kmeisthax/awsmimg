//TODO: Can we unpublish agb and provide encoders through boxed access only?
pub mod agb;

/// Supertrait for encoders and decoders of indexed-color image formats.
pub trait IndexedGraphicsProperties {
    /// Retrieves the size of pattern tiles in this image format.
    /// 
    /// In some hardware indexed color formats, images are separated into a
    /// grid of identically-sized tiles. (Sometimes also referred to as
    /// character cells) Tiles can be placed anywhere on the image's tile grid
    /// and repeated if necessary.
    /// 
    /// For the convenience of image editing, tiled image formats will be
    /// decoded such that every 8x8 tile is placed left to right then top to
    /// bottom on the decoded image. This transformation will be reversed when
    /// encoding images. Encoders and decoders do not need to worry about
    /// performing this transformation themselves: provided image data will
    /// have already been organized linearly.
    /// 
    /// Image formats without tiles should return (0,0).
    fn tile_size(&self) -> (u32, u32);
    
    /// Retrieves the size of a color attribute in this image format.
    /// 
    /// In some hardware indexed color formats, the color palette is specific
    /// to a particular region of the screen - usually an 8x8 or 16x16 square.
    /// All colors within an attribute must be pulled from the same tile. Some
    /// hardware also allows applying other transformations to screen data to
    /// an attribute region of the screen.
    /// 
    /// Image formats without color attributes should return (0,0).
    fn attribute_size(&self) -> (u32, u32);
    
    /// Retrieves the maximum number of colors in a palette.
    /// 
    /// This is the largest index that the format can represent. It does not
    /// imply a limit on the size of palette Vec<u8>s passed to encode_palette.
    fn palette_maxcol(&self) -> u16;
}

pub enum IndexedFormat {
    AGB4,       //4 bits per pixel, packed, arranged row-major in 8x8 tiles
    AGB8Tiled,  //8 bits per pixel, packed, arranged row-major in 8x8 tiles
    AGB8Chunky  //8 bits per pixel, packed, arranged row-major
}

pub fn interpret_indexed_format_name(fmt_given: &str) -> Option<IndexedFormat> {
    let fmt = fmt_given.to_ascii_lowercase();
    
    match fmt.as_ref() {
        "agb4" => Some(IndexedFormat::AGB4),
        "agb8t" => Some(IndexedFormat::AGB8Tiled),
        "agb8c" => Some(IndexedFormat::AGB8Chunky),
        _ => None
    }
}

pub enum DirectFormat {
    AGB16, //16 bits per pixel, packed, RGB5N1, arragned row-major
    NTR16  //16 bits per pixel, packed, RGB5A1, arragned row-major
}

pub fn interpret_direct_format_name(fmt_given: &str) -> Option<DirectFormat> {
    let fmt = fmt_given.to_ascii_lowercase();
    
    match fmt.as_ref() {
        "agb16" => Some(DirectFormat::AGB16),
        "ntr16" => Some(DirectFormat::NTR16),
        _ => None
    }
}