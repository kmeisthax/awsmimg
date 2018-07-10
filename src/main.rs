extern crate argparse;
extern crate image;
extern crate num;

mod asmimg;

use argparse::{ArgumentParser, Store};
use asmimg::encoder::encode_grayscale_image;

fn main() {
    let mut input_filename = "".to_string();
    let mut output_filename = "".to_string();
    let mut format = "".to_string();
    
    {
        let mut ap = ArgumentParser::new();
        
        ap.set_description("Convert a modern image file into a retro format.");
        
        ap.refer(&mut input_filename).add_argument("input", Store, "Name of the modern image file to convert.");
        ap.refer(&mut output_filename).add_argument("output", Store, "Where to store the converted image as.");
        ap.refer(&mut format).add_option(&["--format"], Store, "The format to convert the image into.");
        
        ap.parse_args_or_exit();
    }
    
    println!("Converting {} to {}", input_filename, format);
    
    let img = image::open(input_filename).unwrap();
    
    println!()
}