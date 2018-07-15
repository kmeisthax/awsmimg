use std::io;
use std::io::Write;
use image::{GenericImage, Primitive, Rgba, Pixel};

use asmimg::formats::{IndexedGraphicsProperties, IndexedFormat, DirectFormat};
use asmimg::formats::agb::{AGB4Encoder, AGB8Encoder, AGB16Encoder};
use asmimg::conversion::indexes_from_luma;

/// Represents a struct which can encode color indexes and their palettes into
/// a particular indexed image format.
/// 
/// The format supported by the impl may be tiled, chunky, planar, interleaved,
/// or even compressed; as long as the format ultimately represents some kind
/// of color index into a hardware palette.
pub trait IndexedGraphicsEncoder : IndexedGraphicsProperties {
    /// Given a vector of palette indexes, encode them for the particular
    /// graphics format this encoder supports.
    /// 
    /// The width and height values indicate the shape of the data. data must
    /// always contain width * height elements. If the given graphics format
    /// has a tile size, then the width and height must be multiples of the
    /// width and height of a single tile.
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
    fn encode_palette<T: Primitive>(&mut self, palette: Vec<Rgba<T>>) -> io::Result<()>;
}

/// Given an image and an encoder, encode index data by interpreting the
/// grayscale values of an image as indicies.
/// 
/// The grayscale-image-as-index-data approach is useful because it assigns an
/// unambiguous color to every index, allowing editing of the graphical data
/// using image manipulation tools that don't provide palette editing.
pub fn encode_image_as_indexes<'a, E, I, P, S>(enc: &mut E, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static, E: IndexedGraphicsEncoder + 'a {
    let (width, height) = image.dimensions();
    
    let gdata = indexes_from_luma(image, S::from(enc.palette_maxcol()).unwrap());
    enc.encode_indexes(gdata, width, height)
}

/// Given an image, a writer, and a format description, encode index data by
/// interpreting the grayscale values of an image as indicies.
/// 
/// This function allows access to built-in, private type implementations of
/// these traits. It is currently not possible to access these types through any
/// other means as they are private and IndexedGraphicsEncoder cannot be
/// dynamically dispatched.
pub fn encode_image_as_indexes_with_format<'a, W, I, P, S>(format: IndexedFormat, w: &mut W, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static, W: Write + 'a {
    match format {
        IndexedFormat::AGB4 => encode_image_as_indexes(&mut AGB4Encoder::new(w), image),
        IndexedFormat::AGB8Tiled => encode_image_as_indexes(&mut AGB8Encoder::new_tiled(w), image),
        IndexedFormat::AGB8Chunky => encode_image_as_indexes(&mut AGB8Encoder::new_chunky(w), image)
    }
}

/// Represents a struct which can encode color images into a particular direct
/// color image format.
/// 
/// The format supported by the impl may be tiled, chunky, planar, interleaved,
/// or even compressed; as long as the format ultimately represents some kind
/// of color index into a hardware palette.
pub trait DirectGraphicsEncoder {
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
    fn encode_colors<I, P, S>(&mut self, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static;
}

/// Given an image, a writer, and a format description, encode an image using
/// it's color values to directly determine the color used in the final image.
/// 
/// This function allows access to built-in, private type implementations of
/// these traits. It is currently not possible to access these types through any
/// other means as they are private and DirectGraphicsEncoder cannot be
/// dynamically dispatched.
pub fn encode_image_as_direct_color_with_format<'a, W, I, P, S>(format: DirectFormat, w: &mut W, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static, W: Write + 'a {
    match format {
        DirectFormat::AGB16 => AGB16Encoder::new_agb(w).encode_colors(image),
        DirectFormat::NTR16 => AGB16Encoder::new_ntr(w).encode_colors(image)
    }
}