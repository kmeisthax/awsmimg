extern crate argparse;
extern crate image;
extern crate num;

mod asmimg;

use argparse::{ArgumentParser, Store, StoreFalse, StoreTrue};
use std::fs::{OpenOptions};
use std::io;
use std::io::Seek;
use asmimg::encoder::encode_grayscale_image;
use asmimg::formats::interpret_indexed_format_name;

fn main() -> io::Result<()> {
    let mut input_filename = "".to_string();
    let mut output_filename = "".to_string();
    let mut format = "".to_string();
    let mut truncatemode = true;
    let mut offset = 0u64;
    
    {
        let mut ap = ArgumentParser::new();
        
        ap.set_description("Convert a modern image file into a retro format.");
        
        ap.refer(&mut input_filename).add_argument("input", Store, "Name of the modern image file to convert.");
        ap.refer(&mut output_filename).add_argument("output", Store, "Where to store the converted image as.");
        ap.refer(&mut format).add_option(&["--format"], Store, "The format to convert the image into.");
        ap.refer(&mut truncatemode).add_option(&["--overlay"], StoreFalse, "Overlay encoding result onto existing file. Negates --truncate.")
                                   .add_option(&["--truncate"], StoreTrue, "Erases existing file (if any) before encoding. Negates --overlay.");
        ap.refer(&mut offset).add_option(&["--offset"], Store, "Where to write data to within the target file.");
        
        ap.parse_args_or_exit();
    }
    
    println!("Converting {} to {}", input_filename, output_filename);
    
    let img = image::open(input_filename).unwrap();
    let mut bin = OpenOptions::new().write(true).create(true).truncate(truncatemode).open(output_filename)?;
    let fmt = interpret_indexed_format_name(&format).unwrap();
    
    let orig_length = bin.seek(io::SeekFrom::End(0))?;
    if offset > orig_length {
        //Seeking beyond the end of a file is implementation defined. Hence, we error out
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Proposed offset length exceeds length of file."))
    }
    
    bin.seek(io::SeekFrom::Start(offset))?;
    
    {
        encode_grayscale_image(fmt, &mut bin, &img)
    }
}