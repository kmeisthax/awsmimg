//TODO: Can we unpublish agb and provide encoders through boxed access only?
pub mod agb;

pub enum IndexedFormat {
    AGB4,
    AGB8
}

pub fn interpret_indexed_format_name(fmt_given: &str) -> Option<IndexedFormat> {
    let fmt = fmt_given.to_ascii_lowercase();
    
    match fmt.as_ref() {
        "agb4" => Some(IndexedFormat::AGB4),
        "agb8" => Some(IndexedFormat::AGB8),
        _ => None
    }
}