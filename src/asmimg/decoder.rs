use std::ops::Div;

use std::io;
use std::io::Read;
use image::{GenericImage, ImageBuffer, Primitive, Pixel, LumaA};

use asmimg::formats::{IndexedGraphicsProperties, IndexedFormat};
use asmimg::formats::agb::{AGB4Encoder, AGB8Encoder};
use asmimg::conversion::luma_from_indexes;

pub trait IndexedGraphicsDecoder : IndexedGraphicsProperties {
    /// Decode previously-encoded data into a vector of index data.
    /// 
    /// The given size parameter may be used to limit the amount of image data
    /// decoded. This constitutes an upper bound on how many bytes are allowed
    /// to be read from the decoder's data source. If the decoder's data source
    /// has fewer bytes than allowed by the size parameter, the decoder should
    /// treat the data source's remaining number of bytes as the limiting
    /// factor.
    /// 
    /// If the format being decoded contains size information or a stop symbol,
    /// that information shall constitute a further upper bound on decoding.
    /// Lengths so internally specified should be respected the same as the
    /// size parameter or data underrun conditions.
    /// 
    /// In the event that any aformentioned limitation on decoding causes the
    /// underlying datastream to terminate improperly, the decoder must yield
    /// an error instead of attempting to reconstruct potentially corrupted
    /// data. The meaning of "improper termination" is implementation defined.
    /// Implementations of decoders must take care to ensure that any situation
    /// where data is being misinterpreted, misdecoded, or is incomplete
    /// results in an error rather than invalid data.
    fn decode_indexes<P: Primitive>(&mut self, size: usize) -> io::Result<Vec<P>>;
}

/// Given an image and a decoder, decode index data by interpreting the
/// grayscale values of an image as indicies.
///
/// The grayscale-image-as-index-data approach is useful because it assigns an
/// unambiguous color to every index, allowing editing of the graphical data
/// using image manipulation tools that don't provide palette editing.
pub fn decode_indexes_as_image<'a, E>(enc: &mut E, size: usize, isize: Option<(u32, u32)>) -> io::Result<Box<ImageBuffer<LumaA<u8>, Vec<u8>>>> where E: IndexedGraphicsDecoder + 'a {
    let indexes : Vec<u8> = enc.decode_indexes(size)?;
    let img = luma_from_indexes(indexes, enc.palette_maxcol(), enc.tile_size(), isize);
    match img {
        Some(i) => Ok(i),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, ""))
    }
}

/// Given an image, a writer, and a format description, encode index data by
/// interpreting the grayscale values of an image as indicies.
///
/// This function allows access to built-in, private type implementations of
/// these traits. It is currently not possible to access these types through any
/// other means as they are private and IndexedGraphicsEncoder cannot be
/// dynamically dispatched.
pub fn decode_indexes_as_image_with_format<'a, R>(format: IndexedFormat, r: &mut R, size: usize, imgsize: Option<(u32, u32)>) -> io::Result<Box<ImageBuffer<LumaA<u8>, Vec<u8>>>> where R: Read + 'a {
    match format {
        IndexedFormat::AGB4 => decode_indexes_as_image(&mut AGB4Encoder::new(r), size, imgsize),
        IndexedFormat::AGB8Tiled => decode_indexes_as_image(&mut AGB8Encoder::new_tiled(r), size, imgsize),
        IndexedFormat::AGB8Chunky => decode_indexes_as_image(&mut AGB8Encoder::new_chunky(r), size, imgsize)
    }
}
