use image::{GenericImage, Pixel, Primitive};
use num::NumCast;

/// Convert image data to color indexes using their luminance value.
/// 
/// RGB data will be converted to grayscale. Alpha channels will be discarded.
/// Once converted to luminance data, each individual value will be mapped to
/// an integer within the range [0, maxcol) to produce a final integer value.
pub fn indexes_from_luma<I, P, S>(image: &I, maxcol: S) -> Vec<S>
    where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static {
    
    let (width, height) = image.dimensions();
    let mut out : Vec<S> = Vec::with_capacity(width as usize * height as usize);
    let imgmax = S::max_value();
    let imgmax: i32 = NumCast::from(imgmax).unwrap();
    
    for (_, _, pixel) in image.pixels() {
        let gray = pixel.to_luma();
        
        out.push(gray[0] / S::from(imgmax).unwrap() * maxcol);
    }
    
    out
}