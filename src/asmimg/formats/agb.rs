use asmimg::encoder::{IndexedGraphicsEncoder, DirectGraphicsEncoder};
use asmimg::decoder::IndexedGraphicsDecoder;
use asmimg::tiles::TileChunkIterator;

use std::io;
use std::io::{Write, Read, ErrorKind};
use image::{GenericImage, Primitive, Rgba, Pixel};

/// Encode a series of RGBA colors as palette data.
fn encode_palette<'a, I: Iterator, T: Primitive, W: Write + 'a>(w: &'a mut W, palette: I, use_alpha: bool) -> io::Result<()> where I: Iterator<Item=Rgba<T>> {
    let imgmax = T::max_value();
    let mut out: [u8; 2] = [0, 0];

    for rgba in palette {
        let r : u16 = (rgba[0].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
        let g : u16 = (rgba[1].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
        let b : u16 = (rgba[2].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
        let a : u16 = match use_alpha {
            true => (rgba[3].to_f32().unwrap() / imgmax.to_f32().unwrap()) as u16,
            false => 0
        };
        
        let enc_color: u16 = (a & 0x80) << 8 | (b & 0xF8) << 7 | (g & 0xF8) << 2 | r >> 3;
        
        out[0] = ((enc_color >> 0) & 0xFF) as u8;
        out[1] = ((enc_color >> 8) & 0xFF) as u8;
        w.write(&out)?;
    }

    Ok(())
}

struct ImageRgbaIterator<'a, I, P, S> where I: Iterator<Item=(u32, u32, P)> + 'a, P: Pixel<Subpixel=S> + 'a, S: Primitive + 'a {
    i: &'a mut I
}

impl<'a, I, P, S> ImageRgbaIterator<'a, I, P, S> where I: Iterator<Item=(u32, u32, P)> + 'a, P: Pixel<Subpixel=S> + 'a, S: Primitive + 'a {
    pub fn new(i: &'a mut I) -> ImageRgbaIterator<'a, I, P, S> {
        ImageRgbaIterator {
            i: i
        }
    }
}

impl<'a, I, P, S> Iterator for ImageRgbaIterator<'a, I, P, S> where I: Iterator<Item=(u32, u32, P)> + 'a, P: Pixel<Subpixel=S> + 'a, S: Primitive + 'a {
    type Item = Rgba<S>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.i.next() {
            None => None,
            Some((_, _, p)) => Some(p.to_rgba())
        }
    }
}

/// Encoder/decoder for 4bpp tile patterns for the AGB platform.
pub struct AGB4Encoder<'a, F: 'a> {
    f: &'a mut F,
}

impl<'a, F: 'a> AGB4Encoder<'a, F> {
    pub fn new(file: &'a mut F) -> AGB4Encoder<'a, F> {
        AGB4Encoder {
            f: file
        }
    }
}

impl<'a, F: 'a> IndexedGraphicsEncoder for AGB4Encoder<'a, F> where F: Write {
    fn encode_indexes<P: Primitive>(&mut self, data: Vec<P>, width: u32, _height: u32) -> io::Result<()> {
        let mut out: [u8; 1] = [0];
        
        for tile in TileChunkIterator::new(data, 8, 8, width) {
            for byte in tile.chunks(2) {
                out[0] = byte[0].to_u8().unwrap() & 0x0F | (byte[1].to_u8().unwrap() & 0x0F) << 4;
                self.f.write(&out)?;
            }
        }
        
        Ok(())
    }
    
    fn encode_palette<T: Primitive>(&mut self, palette: Vec<Rgba<T>>) -> io::Result<()> {
        encode_palette(self.f, palette.into_iter(), false)
    }
    
    fn palette_maxcol(&self) -> u16 {
        15
    }
}

impl<'a, F: 'a> IndexedGraphicsDecoder for AGB4Encoder<'a, F> where F: Read {
    fn decode_indexes<P: Primitive>(&mut self, size: usize) -> io::Result<Vec<P>> {
        let mut out = Vec::with_capacity(size * 2);
        let mut buf: [u8; 1] = [0];
        
        for i in 0..size {
            let readcnt = self.f.read(&mut buf)?;
            
            if readcnt < 1 {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "File is shorter than image being decoded"));
            }
            
            out.push(P::from(buf[0] & 0x0F).unwrap());
            out.push(P::from(buf[0] >> 4).unwrap());
        }
        
        Ok(out)
    }
}

/// Encoder for 8bpp tile patterns for the AGB platform.
pub struct AGB8Encoder<'a, W: Write + 'a> {
    w: &'a mut W,
    tsize: u32
}

impl<'a, W:Write + 'a> AGB8Encoder<'a, W> {
    pub fn new_tiled(write: &'a mut W) -> AGB8Encoder<'a, W> {
        AGB8Encoder {
            w: write,
            tsize: 8
        }
    }
    
    pub fn new_chunky(write: &'a mut W) -> AGB8Encoder<'a, W> { 
        AGB8Encoder {
            w: write,
            tsize: 1
        }
    }
}

impl<'a, W:Write> IndexedGraphicsEncoder for AGB8Encoder<'a, W> {
    fn encode_indexes<P: Primitive>(&mut self, data: Vec<P>, width: u32, _height: u32) -> io::Result<()> {
        let mut out: [u8; 64] = [0; 64];
        let tsize = (self.tsize * self.tsize) as usize;
        
        for tile in TileChunkIterator::new(data, self.tsize, self.tsize, width) {
            for (i, byte) in tile.into_iter().enumerate() {
                out[i] = byte.to_u8().unwrap() & 0xFF;
            }
            
            self.w.write(&out[0 .. tsize])?;
        }
        
        Ok(())
    }
    
    fn encode_palette<T: Primitive>(&mut self, palette: Vec<Rgba<T>>) -> io::Result<()> {
        encode_palette(self.w, palette.into_iter(), false)
    }
    
    fn palette_maxcol(&self) -> u16 {
        255
    }
}

pub struct AGB16Encoder<'a, W: Write + 'a> {
    w: &'a mut W,
    allow_ntr_alpha: bool
}

impl<'a, W: Write + 'a> AGB16Encoder<'a, W> {
    pub fn new_agb(write: &'a mut W) -> AGB16Encoder<'a, W> {
        AGB16Encoder {
            w: write,
            allow_ntr_alpha: false
        }
    }
    
    pub fn new_ntr(write: &'a mut W) -> AGB16Encoder<'a, W> {
        AGB16Encoder {
            w: write,
            allow_ntr_alpha: true
        }
    }
}

impl<'a, W: Write> DirectGraphicsEncoder for AGB16Encoder<'a, W> {
    fn encode_colors<I, P, S>(&mut self, image: &I) -> io::Result<()> where I: GenericImage<Pixel=P>, P: Pixel<Subpixel=S> + 'static, S: Primitive + 'static {
        encode_palette(self.w, ImageRgbaIterator::new(&mut image.pixels()), self.allow_ntr_alpha)
    }
}

#[cfg(test)]
mod tests {
    extern crate num;
    extern crate image;
    
    use std::io::Cursor;
    use asmimg::encoder::{IndexedGraphicsEncoder, DirectGraphicsEncoder};
    use asmimg::decoder::IndexedGraphicsDecoder;
    use asmimg::formats::agb::{AGB4Encoder, AGB8Encoder, AGB16Encoder};
    
    #[test]
    fn data4_encode() {
        let src = num::range(0, 64).collect();
        let mut test_out = Cursor::new(Vec::with_capacity(32));
        
        {
            let mut agb4 = AGB4Encoder::new(&mut test_out);

            agb4.encode_indexes(src, 8, 8).unwrap();
        }
        
        let valid_out : Vec<u8> = vec![0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE];
        
        assert_eq!(test_out.get_ref(), &valid_out)
    }
    
    #[test]
    fn data4_decode() {
        let src : Vec<u8> = vec![0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                 0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                 0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                 0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE];
        let mut test_in = Cursor::new(&src);
        let mut agb4 = AGB4Encoder::new(&mut test_in);
        
        let test_out : Vec<u8> = agb4.decode_indexes(src.len()).unwrap();
        let valid_out : Vec<u8> = vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
                                       0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
                                       0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,
                                       0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
        
        assert_eq!(&test_out, &valid_out)
    }
    
    #[test]
    fn data8t_encode() {
        let src = num::range(0, 64).collect();
        let mut test_out = Cursor::new(Vec::with_capacity(64));
        
        {
            let mut agb4 = AGB8Encoder::new_tiled(&mut test_out);

            agb4.encode_indexes(src, 8, 8).unwrap();
        }
        
        let valid_out : Vec<u8> = num::range(0, 64).collect();
        
        assert_eq!(test_out.get_ref(), &valid_out)
    }
    
    #[test]
    fn data8c_encode() {
        let src = num::range(0, 64).collect();
        let mut test_out = Cursor::new(Vec::with_capacity(64));
        
        {
            let mut agb4 = AGB8Encoder::new_chunky(&mut test_out);

            agb4.encode_indexes(src, 8, 8).unwrap();
        }
        
        let valid_out : Vec<u8> = num::range(0, 64).collect();
        
        assert_eq!(test_out.get_ref(), &valid_out)
    }
    
    #[test]
    fn data16_encode() {
        let img = image::ImageBuffer::from_fn(8, 8, |x, y| {
            image::Rgba([(x * 8) as u8, (y as i16 * -8 + 255) as u8, (x * y) as u8, ((x + y) & 0x01 * 255) as u8])
        });
        let mut test_out = Cursor::new(Vec::with_capacity(64));
        
        {
            let mut agb16 = AGB16Encoder::new_agb(&mut test_out);
            
            agb16.encode_colors(&img);
        }
        
        //TODO: Actually round-trip this against an AGB16 decoder (not yet written)
        //This vector was obtained by grabbing some valid-looking output from
        //the code under test and spot-checking a few values against the above
        let valid_out : Vec<u8> = vec![224, 3, 225, 3, 226, 3, 227, 3, 228, 3, 229, 3, 230, 3, 231, 3,
                                       192, 3, 193, 3, 194, 3, 195, 3, 196, 3, 197, 3, 198, 3, 199, 3,
                                       160, 3, 161, 3, 162, 3, 163, 3, 164, 7, 165, 7, 166, 7, 167, 7,
                                       128, 3, 129, 3, 130, 3, 131, 7, 132, 7, 133, 7, 134, 11, 135, 11,
                                       96, 3, 97, 3, 98, 7, 99, 7, 100, 11, 101, 11, 102, 15, 103, 15,
                                       64, 3, 65, 3, 66, 7, 67, 7, 68, 11, 69, 15, 70, 15, 71, 19,
                                       32, 3, 33, 3, 34, 7, 35, 11, 36, 15, 37, 15, 38, 19, 39, 23,
                                       0, 3, 1, 3, 2, 7, 3, 11, 4, 15, 5, 19, 6, 23, 7, 27];
        
        assert_eq!(test_out.get_ref(), &valid_out)
    }
}