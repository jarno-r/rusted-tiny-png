use adler::adler32_slice;
use bitvec::prelude::*;
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fs::File;
use std::io::Write;
use std::{env, mem::size_of, mem::transmute};

fn chunk(chunk_type: &[u8], data: &[u8]) -> Vec<u8> {
    let len = (data.len() as u32).to_be_bytes();

    let mut content = Vec::<u8>::new();
    content.extend(chunk_type);
    content.extend(data);

    let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc.digest();
    digest.update(content.as_slice());
    let crc = digest.finalize();

    let mut chunk = Vec::new();

    chunk.extend(len);
    chunk.extend(content);
    chunk.extend(crc.to_be_bytes());

    chunk
}

fn deflate(data: &[u8]) -> Vec<u8> {
    // Last block of compressed data flag and
    // BTYPE=01 (block with fixed Huffman codes)
    let mut compressed = bitvec![u8, Lsb0; 1,1,0];

    for d in data {
        let value;
        let code = if *d < 144 {
            value = 0b00110000u32 + *d as u32;
            &value.view_bits::<Msb0>()[32 - 8..]
        } else {
            value = 0b110010000u32 + (*d - 144) as u32;
            &value.view_bits::<Msb0>()[32 - 9..]
        };
        compressed.extend(code);
    }

    // Codepoint 256, end-of-block
    compressed.extend([0, 0, 0, 0, 0, 0, 0].map(|x| x == 1));

    Vec::from(compressed.as_raw_slice())
}

fn zlib(data: &[u8], disable_adler: bool) -> Vec<u8> {
    let adler = adler32_slice(data);

    // Compression method CM = 8
    // CINFO = 7 -> 32kb LZ77 window size (not really true in this case)
    // FCHECK = 1, checksum of the two bytes
    // FDICT = 0, no preset dictionary
    // FLEVEL = 0, compression level
    let mut zlib = vec![0x78, 0x01];
    zlib.extend(deflate(data));
    if !disable_adler {
        zlib.extend(adler.to_be_bytes());
    }

    zlib
}

#[repr(C, packed)]
struct IHDR {
    width: u32,
    height: u32,
    bit_depth: u8,
    colour_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

fn main() {
    const USAGE: &str = "Usage: rusted-tiny-png <PNG file> <0-255> [skip-adler-32] [skip-IEND]";

    let mut args = env::args().skip(1);
    let filename = args.next().expect(USAGE);

    let shade = args.next().and_then(|s| s.parse::<u8>().ok()).expect(USAGE);

    let disable_adler = args.next().map(|_| true).unwrap_or(false);

    let disable_iend = args.next().map(|_| true).unwrap_or(false);

    println!("Creating a PNG with gray shade {}", shade);

    // Start with PNG signature
    let mut png = vec![0x89u8, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

    let ihdr = IHDR {
        width: 1u32.to_be(),
        height: 1u32.to_be(),
        bit_depth: 8,
        colour_type: 0,
        compression_method: 0,
        filter_method: 0,
        interlace_method: 0,
    };
    let data = unsafe { transmute::<IHDR, [u8; size_of::<IHDR>()]>(ihdr) };

    // Header chunk.
    png.extend(&chunk("IHDR".as_bytes(), &data));

    // IDAT contains pixel data. Each row is preceded by filter type (0 - no filter).
    png.extend(&chunk(
        "IDAT".as_bytes(),
        &zlib([0, shade].as_slice(), disable_adler),
    ));

    // End of file.
    if !disable_iend {
        png.extend(&chunk("IEND".as_bytes(), [].as_slice()));
    }

    let mut file = File::create(&filename).expect(&format!("Couldn't open file {}", &filename));

    file.write_all(&png).expect("Failed to write to file");

    println!("Raw data:");
    for c in png {
        print!("{:x?} ", c);
    }
}
