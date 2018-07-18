extern crate argparse;
extern crate image;
extern crate num;

mod asmimg;

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};
use std::fs::{OpenOptions};
use std::io;
use std::io::Seek;
use asmimg::decoder::{decode_indexes_as_image_with_format};
use asmimg::formats::{interpret_indexed_format_name, interpret_direct_format_name};

fn main() -> io::Result<()> {
    let mut input_filename = "".to_string();
    let mut output_filename = "".to_string();
    let mut format = "".to_string();
    let mut offset = 0u64;
    let mut size = u64::max_value();

    {
        let mut ap = ArgumentParser::new();

        ap.set_description("Convert retro image data into a modern format.");

        ap.refer(&mut input_filename).add_argument("input", Store, "The retro image data to decode.");
        ap.refer(&mut output_filename).add_argument("output", Store, "Where to store the modern image file.");
        ap.refer(&mut format).add_option(&["--format"], Store, "The format to convert the image from.");
        ap.refer(&mut offset).add_option(&["--offset"], Store, "Where to read data from within the source file.");
        ap.refer(&mut size).add_option(&["--size"], Store, "Maximum amount of data to read from the file.");

        ap.parse_args_or_exit();
    }

    println!("Decoding {} to {}", input_filename, output_filename);

    let mut bin = OpenOptions::new().read(true).open(input_filename)?;
    let orig_length = bin.seek(io::SeekFrom::End(0))?;
    if offset > orig_length {
        //Seeking beyond the end of a file is implementation defined. Hence, we error out
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Proposed offset length exceeds length of file."))
    }
    bin.seek(io::SeekFrom::Start(offset))?;

    let idxfmt = interpret_indexed_format_name(&format).unwrap();
    let img = decode_indexes_as_image_with_format(idxfmt, &mut bin, size as usize, None)?;

    img.save(output_filename)
}
