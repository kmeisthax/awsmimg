use std::io;
use image::{GenericImage, Primitive, Rgb, Pixel};

pub trait IndexedGraphicsEncoder {
    /// Given a vector of palette indexes, encode them for the particular
    /// graphics format this encoder supports.
    /// 
    /// The width and height values indicate the shape of the data. data must
    /// always contain width * height elements. If the given graphics format
    /// has a tile size, then the width and height must be multiples of the
    /// width and height of a single tile.
    /// 
    /// Despite being a DecodingResult, the contents of data will always be
    /// treated as color indicies. If your image is bitmapped graphics, then
    /// it's colors must be mapped to color indexes before encoding. See
    /// asmimg::conversion for functions which extract or generate index data
    /// from a bitmap image.
    /// 
    /// Indexes beyond the maximum number of colors supported by the format
    /// will be truncated. e.g. a 4bpp indexed color format encoder told to
    /// encode the index 22 must instead truncate that index within the range
    /// of 0-15 (via modulo usually) and encode index 6 instead.
    fn encode_indexes<P: Primitive>(&mut self, data: Vec<P>, width: u32, height: u32) -> io::Result<()>;
    
    /// Given a vector of RGB color data, encode each color triplet for the
    /// particular palette format used to colorize the above color indexes.
    /// 
    /// Color palettes longer than the maximum number of colors supported by
    /// the format must not be truncated. If a 4bpp indexed color format
    /// encoder is told to encode a palette consisting of 22 colors, it must
    /// write 22 colors to the palette. Conversely, a palette underflow must
    /// not be padded to the length of a single palette.
    fn encode_palette<T: Primitive>(&mut self, palette: Vec<Rgb<T>>) -> io::Result<()>;
    
    /// Retrieves the maximum number of colors in a palette.
    /// 
    /// This is the limit on how many colors can be represented within a region
    /// of screen space known as the attribute size. It does not imply a limit
    /// on the size of palette Vec<u8>s passed to encode_palette.
    fn palette_maxcol(&self) -> u16;
}

pub trait BitmappedGraphicsEncoder {
    /// Given a vector of color data, encode this data for the particular
    /// graphics format this encoder supports.
    /// 
    /// The width and height values indicate the shape of the data. data must
    /// always contain width * height elements. If the given graphics format
    /// has a tile size, then the width and height must be multiples of the
    /// width and height of a single tile.
    /// 
    /// Indexed color images cannot be encoded by this interface. Attempting to
    /// do so will result in Err.
    /// 
    /// Graphics formats with lower bit depths must convert higher bit-depth
    /// images by rounding to the nearest neighbor and not by any other method.
    /// In particular, dithering is not permitted.
    fn encode_colors<I, P, S>(&mut self, image: I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static;
}