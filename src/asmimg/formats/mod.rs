mod agb;

use std::io;
use std::io::Write;
use image::{GenericImage, Pixel, Primitive};

use asmimg::encoder::IndexedGraphicsEncoder;
use asmimg::formats::agb::AGB4Encoder;
use asmimg::conversion::indexes_from_luma;

pub enum IndexedFormat {
    AGB4
}

/// Given an image and an encoder, encode the image by treating it's color
/// values as color indexes.
pub fn encode_grayscale_image<'a, W, I, P, S>(format: IndexedFormat, mut w: &mut W, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static, W: Write + 'a {
    let mut enc = match format {
        IndexedFormat::AGB4 => AGB4Encoder::new(w)
    };
    
    let (width, height) = image.dimensions();
    let gdata = indexes_from_luma(image, S::from(enc.palette_maxcol()).unwrap());
    enc.encode_indexes(gdata, width, height)
}