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
    let imgmax: f32 = NumCast::from(imgmax).unwrap();
    let maxcol_adj: f32 = NumCast::from(maxcol - S::from(1).unwrap()).unwrap();
    
    for (_, _, pixel) in image.pixels() {
        let gray = pixel.to_luma()[0].to_f32().unwrap();
        out.push(S::from(gray / imgmax * maxcol_adj).unwrap());
    }
    
    out
}