use std::io;
use image::Primitive;

use asmimg::formats::IndexedGraphicsProperties;

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