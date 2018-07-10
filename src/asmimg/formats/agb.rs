use asmimg::encoder::IndexedGraphicsEncoder;
use asmimg::tiles::TileChunkIterator;

use std::io;
use std::io::Write;
use image::{Primitive, Rgb};

/// Encoder for 4bpp tile patterns for the AGB platform.
pub struct AGB4Encoder<'a, W: Write + 'a> {
    w: &'a mut W,
}

impl<'a, W:Write + 'a> AGB4Encoder<'a, W> {
    pub fn new(write: &'a mut W) -> AGB4Encoder<'a, W> {
        AGB4Encoder {
            w: write
        }
    }
}

impl<'a, W:Write> IndexedGraphicsEncoder for AGB4Encoder<'a, W> {
    fn encode_indexes<P: Primitive>(&mut self, data: Vec<P>, width: u32, _height: u32) -> io::Result<()> {
        let mut out: [u8; 1] = [0];
        
        for tile in TileChunkIterator::new(data, 8, 8, width) {
            for byte in tile.chunks(2) {
                out[0] = byte[0].to_u8().unwrap() & 0x0F | (byte[1].to_u8().unwrap() & 0x0F) << 4;
                self.w.write(&out)?;
            }
        }
        
        Ok(())
    }
    
    fn encode_palette<T: Primitive>(&mut self, palette: Vec<Rgb<T>>) -> io::Result<()> {
        let imgmax = T::max_value();
        let mut out: [u8; 2] = [0, 0];
        
        for rgb in palette {
            let r : u16 = (rgb[0].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
            let g : u16 = (rgb[1].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
            let b : u16 = (rgb[2].to_f32().unwrap() / imgmax.to_f32().unwrap() * 255f32) as u16;
            let enc_color: u16 = b & 0xF8 << 7 | g & 0xF8 << 2 | r >> 3;
            out[0] = ((enc_color >> 0) & 0xFF) as u8;
            out[1] = ((enc_color >> 8) & 0xFF) as u8;
            self.w.write(&out)?;
        }
        
        Ok(())
    }
    
    fn palette_maxcol(&self) -> u16 {
        16
    }
}

#[cfg(test)]
mod tests {
    extern crate num;
    
    use std::io::Cursor;
    use asmimg::encoder::IndexedGraphicsEncoder;
    use asmimg::formats::agb::AGB4Encoder;
    
    #[test]
    fn data_encode() {
        let src = num::range(0, 64).collect();
        let mut test_out = Cursor::new(Vec::with_capacity(10));
        
        {
            let mut agb4 = AGB4Encoder::new(&mut test_out);

            agb4.encode_indexes(src, 8, 8);
        }
        
        let valid_out : Vec<u8> = vec![0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
                                     0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE];
        
        assert_eq!(test_out.get_ref(), &valid_out)
    }
}