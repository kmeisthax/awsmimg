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