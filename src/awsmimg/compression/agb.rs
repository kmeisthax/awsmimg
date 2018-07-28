use std::io;
use std::io::{Read, Write};
use std::cmp::{Ord, Ordering, min};
use std::collections::binary_heap::BinaryHeap;

#[derive(Copy, Clone)]
enum AGBHuffmanNode {
    Branch(usize), //Index of next tree node to read a bit from.
    Leaf(u8)       //End of the Huffman tree - output this compressed symbol.
}

type AGBHuffmanTree = (AGBHuffmanNode, AGBHuffmanNode);

/// Implements a decompression filter for reading compressed graphics data.
/// 
/// Reading from this struct will cause Huffman decompression to occur as
/// explained on GBATEK:
/// 
/// https://problemkaputt.de/gbatek.htm#biosdecompressionfunctions
/// 
/// Data header must be present and valid; failing to provide such a header will
/// cause read operations to fail with an InvalidData error. Errors raised by
/// the underlying Read object will pass through this object.
/// 
/// AGB compressed data formats contain internal size information that
/// constitutes a limit on how many bytes can be decompressed from the reader.
/// This read filter will refrain from providing 
struct AGBHuffmanDecompressor<'a, R: Read + 'a> {
    //DATA SOURCE
    r: &'a mut R,
    
    //DECODED AGBHuffman HEADER
    bits_per_symbol: u8, //AKA "Data Size".
    header_type: u8, //Must always be 2. Read from header.
    internal_size: u32, //Number of bytes in decompressed datastream.
    
    //DECODED AGBHuffman TREE
    tree: Vec<AGBHuffmanTree>, //Huffman tree to decompress with.
    
    //INTERNAL DECOMPRESSION STATE
    initialized: bool,
    decompressed_cnt: usize, //Number of bytes decompressed so far.
    bitbuffer: u32, //Remaining already-read bits
    bitbuffer_len: u8, //Number of valid bits remaining in the buffer
}

impl <'a, R: Read + 'a> AGBHuffmanDecompressor<'a, R> {
    fn new(r: &'a mut R) -> AGBHuffmanDecompressor<'a, R> {
        AGBHuffmanDecompressor {
            r: r,
            bits_per_symbol: 0,
            header_type: 2,
            internal_size: 0,
            tree: Vec::new(),
            initialized: false,
            decompressed_cnt: 0,
            bitbuffer: 0,
            bitbuffer_len: 0
        }
    }
    
    fn read_huffman_header(&mut self) -> io::Result<()> {
        let mut hdr = [0u8; 4];
        let readbytes = self.r.read(&mut hdr)?;
        
        if readbytes < hdr.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "The AGB Huffman general header extends past the end of the file."));
        }
        
        //AGB is little endian so I THINK this works!?
        self.bits_per_symbol = hdr[0] & 0x0Fu8;
        self.header_type = hdr[0] >> 4;
        self.internal_size = ((hdr[3] as u32) << 16) | ((hdr[2] as u32) << 8) | (hdr[1] as u32);
        
        if self.header_type != 2 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "This is not AGB Huffman data."))
        }
        
        Ok(())
    }
    
    fn read_huffman_tree(&mut self) -> io::Result<()> {
        let mut hdr = [0u8; 1];
        let readbytes = self.r.read(&mut hdr)?;
        
        if readbytes < hdr.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "The AGB Huffman tree header extends past the end of the file."));
        }
        
        let treesize = ((hdr[0] + 1) * 2).into();
        let mut rawtree = Vec::with_capacity(treesize);
        rawtree.resize(treesize, 0);
        
        let readbytes2 = self.r.read(&mut rawtree.get_mut(0..treesize).unwrap())?;
        
        if readbytes < rawtree.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "The AGB Huffman tree data extends past the end of the file."));
        }
        
        self.tree.resize(treesize / 2, (AGBHuffmanNode::Leaf(0), AGBHuffmanNode::Leaf(0)));
        
        self.read_huffman_tree_internal(&rawtree, 0, false, false)?;
        
        Ok(())
    }
    
    /// Internal recursive function to decode AGBHuffmanTree data.
    fn read_huffman_tree_internal(&mut self, rawtree: &Vec<u8>, offset: usize, lnode_leaf: bool, rnode_leaf: bool) -> io::Result<()> {
        let rawnode = rawtree.get(offset..offset+1).unwrap();
        let mut lnode : AGBHuffmanNode;
        let mut rnode : AGBHuffmanNode;
        
        match lnode_leaf {
            true => {
                lnode = AGBHuffmanNode::Leaf(rawnode[0]);
            },
            false => {
                lnode = AGBHuffmanNode::Branch((offset / 2) + ((rawnode[0] & 0x3F) + 1) as usize);
                self.read_huffman_tree_internal(rawtree, (offset & 0xFE) + ((rawnode[0] & 0x3F) * 2) as usize + 2, rawnode[0] & 0x80 == 0x80, rawnode[0] & 0x40 == 0x40)?;
            }
        }
        
        match rnode_leaf {
            true => {
                rnode = AGBHuffmanNode::Leaf(rawnode[1]);
            },
            false => {
                rnode = AGBHuffmanNode::Branch((offset / 2) + ((rawnode[1] & 0x3F) + 1) as usize);
                self.read_huffman_tree_internal(rawtree, (offset & 0xFE) + ((rawnode[1] & 0x3F) * 2) as usize + 2, rawnode[1] & 0x80 == 0x80, rawnode[1] & 0x40 == 0x40)?;
            }
        }
        
        self.tree[offset / 2] = (lnode, rnode);
        
        Ok(())
    }
    
    /// Attempts to fill the internal Huffman data stream (bitbuffer) with data.
    /// 
    /// Under normal circumstances, after calling this function the bitbuffer
    /// will contain at least 24 bits of data. Up to 7 bits of data will be left
    /// empty as we cannot read less than 8 bits at a time from the file.
    /// 
    /// If the underlying Read reaches an end-of-file condition or returns an
    /// error, no change will be made to the current size of the bitbuffer.
    fn fill_bit_buffer(&mut self) -> io::Result<()> {
        let bits_needed = 32 - self.bitbuffer_len;
        let bytes_needed : usize = (bits_needed / 8).into();
        let mut buf = [0u8; 4];
        let mut bytes_read = 0;
        
        if (bytes_needed == 0) {
            return Ok(())
        }
        
        bytes_read = self.r.read(&mut buf[0..bytes_needed])?;
        
        if (bytes_read == 0) {
            return Ok(())
        }
        
        for i in 0..bytes_read {
            self.bitbuffer = self.bitbuffer | (buf[i] << (i * 8 + self.bitbuffer_len as usize)) as u32;
        }
        
        self.bitbuffer_len += (bytes_read * 8) as u8;
        
        Ok(())
    }
    
    /// Get the next bit from the internal bitbuffer, yielding None if that is
    /// not possible.
    fn get_next_bit(&mut self) -> io::Result<u8> {
        //Get a bit from the bitbuffer
        if (self.bitbuffer_len < 1) {
            self.fill_bit_buffer()?;
        }

        //Raise error if we really can't get more bits
        if (self.bitbuffer_len < 1) {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "The AGB Huffman datastream ended before we could finish decompressing."));
        }
        
        let nextbit = self.bitbuffer & 0x01;
        self.bitbuffer_len -= 1;
        self.bitbuffer = self.bitbuffer >> 1;
        
        Ok(nextbit as u8)
    }
}

impl <'a, R: Read + 'a> Read for AGBHuffmanDecompressor<'a, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if (self.initialized == false) {
            self.read_huffman_header()?;
            self.read_huffman_tree()?;
        }
        
        let decomp_bytes_this_round = min(buf.len(), self.internal_size as usize - self.decompressed_cnt);
        
        for i in 0..decomp_bytes_this_round {
            buf[i] = 0;

            let mut current_huffman_node = self.tree.get(0).unwrap().clone();
            let symbols_per_byte = 8 / self.bits_per_symbol;
            
            for j in 0..symbols_per_byte {
                let shift = j * self.bits_per_symbol;
                let mask : u8 = 0xFF << 8 - shift;
                
                loop {
                    let nextbit = self.get_next_bit()?;
                    let node = match nextbit {
                        0 => current_huffman_node.0,
                        _ => current_huffman_node.1
                    };

                    match node {
                        AGBHuffmanNode::Branch(k) => {
                            current_huffman_node = self.tree.get(k as usize).unwrap().clone();
                        },
                        AGBHuffmanNode::Leaf(d) => {
                            buf[i] = buf[i] | (d & mask) << shift;
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(decomp_bytes_this_round)
    }
}

/// Type alias for a weight referencing a node ID in a memory arena.
///
/// To be used with a binary heap
#[derive(Eq, PartialEq)]
struct AGBHuffmanNodeWeight {
    freq: u32,
    pos: Option<usize>,
    symbol: Option<u8>
}

impl PartialOrd for AGBHuffmanNodeWeight {
    /// Inverted order to force binary heap to be a min heap
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AGBHuffmanNodeWeight {
    /// Inverted order to force binary heap to be a min heap
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

/// Implementation of an AGBHuffman compressor as a Write filter.
///
/// Usage considerations: AGBHuffmanCompressor does not write data to the sink
/// until it is flushed. Flushing the compressor constitutes compressing and
/// writing all compressed data plus the header to the underlying file. This is
/// an additional semantic burden on Write.flush but was considered preferrable
/// to compressing on Drop.
struct AGBHuffmanCompressor<'a, W: Write + 'a> {
    // DATA SINK
    w: &'a mut W,

    // COMPRESSION PARAMETERS
    bits_per_symbol: u8,

    // INTERNAL COMPRESSION STATE
    data: Vec<u8>,
    tree: Vec<AGBHuffmanTree>,
    frequency: Vec<u32>,
}

impl<'a, W: Write + 'a> AGBHuffmanCompressor<'a, W> {
    pub fn new(w: &'a mut W, bits_per_symbol: u8) -> AGBHuffmanCompressor<'a, W> {
        let max_symbols = 2_usize.pow(bits_per_symbol.into());

        AGBHuffmanCompressor {
            w: w,
            bits_per_symbol: bits_per_symbol,
            data: Vec::new(),
            tree: Vec::with_capacity(max_symbols),
            frequency: Vec::with_capacity(max_symbols),
        }
    }

    fn serialize_huffman_tree(&self, tree: &AGBHuffmanTree) -> Vec<u8> {
        let mut treenode = vec![0u8; 2];

        //TODO: How do we allocate space to write child nodes in?

        match tree.0 {
            AGBHuffmanNode::Branch(i) => {
                let child0bit = match self.tree[i].0 {
                    AGBHuffmanNode::Branch(j) => 0x80,
                    AGBHuffmanNode::Leaf(j) => 0x00,
                };
                let child1bit = match self.tree[i].1 {
                    AGBHuffmanNode::Branch(j) => 0x40,
                    AGBHuffmanNode::Leaf(j) => 0x00,
                };

                treenode[0] = 0u8 | child0bit | child1bit;
                treenode.extend(self.serialize_huffman_tree(&self.tree[i]));
            },
            AGBHuffmanNode::Leaf(i) => {
                treenode[0] = i;
            }
        };

        match tree.1 {
            AGBHuffmanNode::Branch(i) => {
                let child0bit = match self.tree[i].0 {
                    AGBHuffmanNode::Branch(j) => 0x80,
                    AGBHuffmanNode::Leaf(j) => 0x00,
                };
                let child1bit = match self.tree[i].1 {
                    AGBHuffmanNode::Branch(j) => 0x40,
                    AGBHuffmanNode::Leaf(j) => 0x00,
                };
                let childpos = treenode.len() / 2 - 1;

                treenode[1] = (childpos & 0x3F) as u8 | child0bit | child1bit;
                treenode.extend(self.serialize_huffman_tree(&self.tree[i]));
            },
            AGBHuffmanNode::Leaf(i) => {
                treenode[1] = i;
            }
        };

        treenode
    }

    fn write_huffman_header(&mut self, rootnode: usize) -> io::Result<()> {
        let mut hdr = [0u8; 5];

        hdr[0] = ((self.bits_per_symbol & 0x0F) | 0x20) as u8;
        hdr[1] = (self.data.len() & 0xFF) as u8;
        hdr[2] = ((self.data.len() >> 8) & 0xFF) as u8;
        hdr[3] = ((self.data.len() >> 16) & 0xFF) as u8;

        //Huffman data
        let hdata = self.serialize_huffman_tree(&self.tree[rootnode]);

        //Huffman header
        hdr[4] = (hdata.len() / 2 - 1) as u8;

        let written = self.w.write(&hdr)?;
        if written < 5 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Couldn't write complete AGB compressed graphics header"))
        }

        let written2 = self.w.write(&hdata)?;
        if written2 < hdata.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Couldn't write complete AGB huffman tree"))
        }

        Ok(())
    }
}

impl<'a, W: Write + 'a> Write for AGBHuffmanCompressor<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);

        for byte in buf {
            let symbols_per_byte = 8 / self.bits_per_symbol;
            for i in 0..symbols_per_byte {
                let shift = i * self.bits_per_symbol;
                let mask : u8 = 0xFF << 8 - shift;
                let symbol = ((byte & mask) >> shift) as usize;

                if (self.frequency.len() < symbol) {
                    self.frequency.resize(symbol, 0);
                }

                self.frequency[symbol] += 1;
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        // Construct a huffman tree and compressed datastream.
        let mut heap = BinaryHeap::new();

        for (i, frequency) in self.frequency.iter().enumerate() {
            if *frequency == 0 {
                continue;
            }

            heap.push(AGBHuffmanNodeWeight{freq: *frequency, pos: None, symbol: Some(i as u8)});
        }

        //By the way this heap works, the last node processed is the root node.
        let mut lastnode = 0;

        while let Some(AGBHuffmanNodeWeight{freq: freq1, pos: pos1, symbol: sym1}) = heap.pop() {
            match heap.pop() {
                Some(AGBHuffmanNodeWeight{freq: freq2, pos: pos2, symbol: sym2}) => {
                    //There are two weights remaining in the queue - join the
                    //lesser node to the greater one.

                    let leftbranch = match pos1 {
                        Some(p) => {
                            AGBHuffmanNode::Branch(p)
                        },
                        None => {
                            AGBHuffmanNode::Leaf(sym1.unwrap())
                        }
                    };

                    let rightbranch = match pos2 {
                        Some(p) => {
                            AGBHuffmanNode::Branch(p)
                        },
                        None => {
                            AGBHuffmanNode::Leaf(sym2.unwrap())
                        }
                    };

                    let newpos = self.tree.len();
                    self.tree.push((leftbranch, rightbranch));
                    heap.push(AGBHuffmanNodeWeight{freq: freq1+freq2, pos: Some(newpos), symbol: None});
                    lastnode = newpos;
                },
                None => {
                    break;
                }
            }
        }

        // At this point self.tree is populated with an ostensibly completed
        // Huffman tree, and we now need to write it out in the order the AGB
        // BIOS expects.



        // Now we need to actually use our tree to encode the data!

        // Flush the underlying file.
        self.w.flush()?;
        Ok(())
    }
}
