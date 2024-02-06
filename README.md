# rusted-tiny-png
Generate the smallest possible PNG in Rust.

The project is based on [this post by Evan Hanh](https://evanhahn.com/worlds-smallest-png/). It is essentially a programming exercise in [Rust](https://www.rust-lang.org/).

The smallest valid PNG is 67 bytes. Evan describes a file that has 1 bit pixel depth, i.e. the pixel is black or white. However, it is possible to achieve the same file size with 8 bit grayscale using any intensity. 

The file size can be further reduced by omitting parts that are not required to decode the image data. The compressed pixel data contains a checksum. Removing the checksum does not prevent decompressing the image data, but reduces the file size by 4 bytes, down to 63 bytes total. 

The file size can be further reduced by removing the end-of-file indicator or the IEND chunk, which makes for another 12 bytes, bringing the file size down to 51 bytes.

Another extra 5 bytes can be shaved off by truncating the pixel data in the IDAT chunk, leaving out the last null byte of the compressed pixel data and the chunk CRC-32. The last null byte of the deflate stream is part of encoding the end-of-block marker. That can be cut short, because the decoder knows that the stream ends anyway. And omitting the CRC-32 checksum obviously has no impact on the pixel data itself.

Technically a file that omits the above-mentioned parts is not a valid PNG file, but none of the applications I have tested seem to care.

## Usage:
  
    rusted-tiny-png <PNG file> <0-255> [skip-adler-32] [skip-IEND] [truncate-IDAT]

<img src="images/tiny.png" width="20"/>
Create a 67 byte tiniest valid PNG with middle grey:

    rusted-tiny-png tiny.png 127

<img src="images/tinier.png" width="20"/>
Create a 63 byte PNG without Adler-32 checksum in light grey:

    rusted-tiny-png tinier.png 211 d

<img src="images/tiniest.png" width="20"/>
Create a 51 byte PNG without Adler-32 checksum and IEND chunk in black:

    rusted-tiny-png tiniest.png 0 d d

<img src="images/tiniestest.png" width="20"/>
Create a 46 byte PNG with truncated IDAT and removed IEND chunk in light greyish:

    rusted-tiny-png tiniestest.png 200 d d d
