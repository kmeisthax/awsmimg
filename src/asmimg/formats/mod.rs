//TODO: Can we unpublish agb and provide encoders through boxed access only?
pub mod agb;

pub enum IndexedFormat {
    AGB4,       //4 bits per pixel, packed, arranged row-major in 8x8 tiles
    AGB8Tiled,  //8 bits per pixel, packed, arranged row-major in 8x8 tiles
    AGB8Chunky  //4 bits per pixel, packed, arranged row-major
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