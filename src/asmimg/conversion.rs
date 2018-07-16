use image::{GenericImage, Pixel, Primitive, ImageBuffer, LumaA};
use num::NumCast;
use std::ops::Div;

/// Given an image, produce a stream of index data to encode by interpreting
/// the grayscale values of the image as indexes.
/// 
/// The given tile size will be used to separate incoming pixels into tiles.
/// asmimg convention is to display tiles from left-to-right, top-to-bottom
/// within an image.
///
/// RGB data will be converted to grayscale. Once converted to luminance data,
/// each individual value will be mapped to an integer within the range
/// [0, maxcol) to produce a final integer value. Alpha values within the image
/// with a value of zero will be ignored for the purposes of determining the
/// size of the data to be converted. When preoparing an image whose dimensions
/// do not divide cleanly into the tile count, you may add "blank" tiles
/// consisting of transparent pixels to indicate that they should not be
/// encoded.
///
/// Do not place transparent pixels in places where a further non-transparent
/// pixel would cause the length of the converted data to cover the transparent
/// pixel. In such cases, the value of that pixel in the encoded data stream is
/// implementation-defined.
pub fn indexes_from_luma<I, P, S>(image: &I, maxcol: S, tsize: (u32, u32)) -> Vec<S>
    where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static {
    
    let (width, height) = image.dimensions();
    let (tw, th) = tsize;
    let mut out : Vec<S> = Vec::with_capacity(width as usize * height as usize);
    let imgmax = S::max_value();
    let imgmax: f32 = NumCast::from(imgmax).unwrap();
    let maxcol_adj: f32 = NumCast::from(maxcol).unwrap();
    
    let tlen = tw * th;
    
    for (ix, iy, pixel) in image.pixels() {
        let la = pixel.to_luma_alpha();
        let gray = la[0].to_f32().unwrap();
        let alpha = la[1].to_u8().unwrap();
        
        let tx = ix / tw;
        let px = ix % tw;
        let ty = iy / tw;
        let py = iy % tw;
        
        let itile = ty * (width / tw) + tx;
        let outidx = (itile * tlen + py * tw + px) as usize;
        
        if outidx > out.len() && alpha != 0u8 {
            out.resize(outidx, S::from(0u8).unwrap());
        }
        
        out[outidx] = S::from(gray / imgmax * maxcol_adj).unwrap();
    }

    out
}

/// Given a stream of decoded index data, produce an image representing the
/// data with color indicies represented as grayscale values and each tile
/// placed left-to-right in the image.
/// 
/// The returned image size will be equal to isize if provided. Otherwise,
/// this function will determine an appropriate image size. In either case,
/// the image size must be a multiple of the tile size for this function to
/// return a valid image. The amount of indexes in data must be a multiple of
/// the tile size as well.
/// 
/// Grayscale values of the resulting image will be mapped to 
/// 
/// As a convenience for image editors, the number of tiles the image size can
/// fit is allowed to deviate from the number of tiles in data. Parts of the
/// image not holding decoded index data will instead be fully transparent
/// pixels. As a result, the pixel format of returned images will be locked to
/// LumaA pixels.
pub fn luma_from_indexes<Pr>(data: Vec<Pr>, maxcol: Pr, tsize: (u32, u32), isize: Option<(u32, u32)>) -> Option<impl GenericImage> where Pr: Primitive, u8: Div<Pr> {
    let mut iw;
    let mut ih;
    let (tw, th) = tsize;
    let tstride = tw * th;
    let tcount = data.len() as u32 / tstride;
    
    //Data length must be cleanly divided by the length of a single tile.
    if tcount * tstride != data.len() as u32 {
        return None;
    }
    
    match isize {
        Some((w, h)) => {
            iw = w;
            ih = h;
        },
        None => {
            iw = (tcount as f32).sqrt().ceil() as u32 * tw;
            ih = ((tcount as f32) / (iw / tw) as f32).ceil() as u32;
        }
    };
    
    //Image size must cleanly divide by tile size.
    if (iw % tw != 0) || (ih % th != 0) {
        return None;
    }
    
    let maxcol : f32 = NumCast::from(maxcol).unwrap();
    let colscale : f32 = 255f32 / maxcol;
    
    //TODO: What if we have a format that needs more than 8 bits of precision?
    let mut out : ImageBuffer<LumaA<u8>, Vec<u8>> = ImageBuffer::from_fn(iw, ih, |x, y| {
        let tx = x / tw; // tile units
        let ty = y / th;
        
        let px = x % tw; // pixel units
        let py = y % th;
        
        let tileid = ty * (iw / tw) + tx;
        let tilepx = px * tw + py;
        let tileidx : usize = NumCast::from(tileid * tstride + tilepx).unwrap();
        
        if tileidx > data.len() {
            LumaA([0u8, 0u8])
        } else {
            let tileval : f32 = NumCast::from(data[tileidx]).unwrap();
            LumaA([NumCast::from(tileval * colscale).unwrap(), 255u8])
        }
    });
    
    Some(out)
}
