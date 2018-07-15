use image::{GenericImage, Pixel, Primitive, ImageBuffer, LumaA};
use num::NumCast;
use std::ops::Div;

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
    let maxcol_adj: f32 = NumCast::from(maxcol).unwrap();
    
    for (_, _, pixel) in image.pixels() {
        let gray = pixel.to_luma()[0].to_f32().unwrap();
        out.push(S::from(gray / imgmax * maxcol_adj).unwrap());
    }
    
    out
}

pub struct TileChunkIterator<T> {
    src: Vec<T>,
    /// Width of a single tile chunk.
    tw: u32,
    /// Height of a single tile chunk.
    th: u32,
    /// Length of a single line of data in src. (Usually image width)
    stride: u32,
    /// Horizontal position of the next tile chunk to return.
    x: u32,
    /// Vertical position of the next tile chunk to return.
    y: u32,
}

impl<T> TileChunkIterator<T> {
    pub fn new(src: Vec<T>, tw: u32, th: u32, stride: u32) -> TileChunkIterator<T> {
        TileChunkIterator {
            src: src, tw: tw, th: th, stride: stride, x: 0, y: 0
        }
    }
}

impl<T> Iterator for TileChunkIterator<T> where T: Clone {
    type Item = Vec<T>;
    
    fn next(&mut self) -> Option<Vec<T>> {
        let mut x2 = self.x + self.tw;
        let mut y2 = self.y + self.th;
        
        //If the next tile is off the right side of the image, go down a row
        if x2 > self.stride {
            self.x = 0;
            self.y = y2;
            
            x2 = self.x + self.tw;
            y2 = self.y + self.th;
        }
        
        //If the next tile is off the bottom of the image, we're done
        if (y2 * self.stride) as usize > self.src.len() {
            return None
        }
        
        let mut v = Vec::with_capacity(self.tw as usize * self.th as usize);
        
        for j in self.y..y2 {
            for i in self.x..x2 {
                v.push(self.src[(j * self.stride + i) as usize].clone());
            }
        }
        
        self.x += self.tw;
        
        Some(v)
    }
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
            iw = 0;
            ih = 0;
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