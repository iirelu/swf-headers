//! A library for reading data from the header of a .swf file

#![warn(missing_docs)]

extern crate byteorder;
extern crate flate2;
extern crate lzma;

mod decoded_swf;
mod bit_range;
mod error;

use std::fs::File;
use std::path::Path;

pub use decoded_swf::DecodedSwf;
pub use bit_range::BitRange;
pub use error::Error;

use byteorder::{LittleEndian, ReadBytesExt};

/// An enum representing all the valid signatures of a swf file
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Signature {
    /// The signature is FWS, meaning an uncompressed swf file.
    Uncompressed,
    /// The signature is CWS, meaning a zlib-compressed swf file.
    ZlibCompressed,
    /// The signature is ZWS, meaning an LZMA-compressed swf file.
    LzmaCompressed
}

/// A struct containing all the parsed headers of a swf file.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SwfHeaders {
    signature: Signature,
    version: u8,
    file_length: u32,
    width: u32,
    height: u32,
    frame_rate: u16,
    frame_count: u16
}

impl SwfHeaders {
    /// Opens a swf file and parses its headers, returning the header struct along with
    /// a readable DecodedSwf if you wish to continue parsing the file.
    pub fn open<T: AsRef<Path>>(path: T) -> Result<(Self, DecodedSwf), Error> {
        Self::read_from(try!(File::open(path)))
    }

    /// Takes a swf file and parses its headers, returning the header struct along with
    /// a readable DecodedSwf if you wish to continue parsing the file.
    pub fn read_from(mut file: File) -> Result<(Self, DecodedSwf), Error> {
        // SWF header strcture overview:
        // Everything is little endian.
        //
        // Signature: u8. Either 'F', 'C', or 'Z' for uncompressed, zlib, or LZMA respectively
        // Magic number: u8. Always 0x57 ('W')
        // Magic number: u8. Always 0x53 ('S')
        // Version: u8
        // File length: u32.
        // Frame size: ???. Here be monsters. A variable-length RECT as defined by the spec
        // Framerate: Says its u16, which is a lie as it's actually an 8.8 fixed point value
        // Frame count: u16

        // Get the signature
        let sig = match try!(file.read_u8()) as char {
            'F' => Signature::Uncompressed,
            'C' => Signature::ZlibCompressed,
            'Z' => Signature::LzmaCompressed,
            _ => return Err(Error::NotSwf)
        };

        // Verify that the magic numbers are correct
        match (try!(file.read_u8()), try!(file.read_u8())) {
            (0x57, 0x53) => {},
            _ => return Err(Error::NotSwf)
        }

        // Get the version
        let version = try!(file.read_u8());
        // Get the file length
        let file_length = try!(file.read_u32::<LittleEndian>());

        // From this point on (the 8th byte), the rest of the file will be likely compressed, so
        // we have to work with a decoded copy.
        let mut decoded = try!(DecodedSwf::decompress(file, sig));

        // The logic for this is painful, so it'll be in its own function.
        let (width, height) = try!(parse_rect(&mut decoded));

        // The frame rate is stored in the header as a fixed-point number. Unless it turns out that
        // decimal points in frame rates are common, we won't bother dealing with it.
        let frame_rate_lower = try!(decoded.read_u8());
        let frame_rate_upper = try!(decoded.read_u8());
        if frame_rate_lower != 0 {
            panic!("Decimal points in frame rates not yet supported");
        }
        let frame_rate = frame_rate_upper as u16;

        let frame_count = try!(decoded.read_u16::<LittleEndian>());

        Ok((SwfHeaders {
            signature: sig,
            version: version,
            file_length: file_length,
            width: width,
            height: height,
            frame_rate: frame_rate,
            frame_count: frame_count
        }, decoded))
    }
    /// Returns the signature as an enum representing all valid values.
    pub fn signature(&self) -> Signature {
        self.signature
    }
    /// Returns the version number.
    pub fn version(&self) -> u8 {
        self.version
    }
    /// Returns the uncompressed total file length in bytes.
    pub fn file_length(&self) -> u32 {
        self.file_length
    }
    /// Returns the dimensions in twips (the measurement unit flash uses, 1/20th of a pixel).
    pub fn dimensions_twips(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    /// Returns the dimensions in pixels (converted from twips, sometimes losing accuracy).
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width / 20, self.height / 20)
    }
    /// Returns the frame rate (note: does not yet handle fractional framerates).
    pub fn frame_rate(&self) -> u16 {
        self.frame_rate
    }
    /// Returns the frame count.
    pub fn frame_count(&self) -> u16 {
        self.frame_count
    }
}

fn parse_rect<T: ReadBytesExt>(file: &mut T) -> Result<(u32, u32), Error> {
    let first_byte = try!(file.read_u8());
    let nbits = ((first_byte >> 3) & 0b0001_1111) as u32;
    let nbytes = (5 + nbits * 4) / 8; // ?

    let mut bytes = Vec::new();
    bytes.push(first_byte);

    for _ in 0..nbytes {
        bytes.push(try!(file.read_u8()));
    }

    let width = bytes.get_bit_range(5+nbits..5+nbits*2);
    let height = bytes.get_bit_range(5+nbits*3..5+nbits*4);

    Ok((width, height))
}


#[cfg(test)]
mod tests {
    use super::*;

    // See tests/README.md for more information about these tests

    #[test]
    fn test_245() {
        let (headers, _) = SwfHeaders::open("tests/245.swf").unwrap();
        assert_eq!(headers.signature(), Signature::ZlibCompressed);
        assert_eq!(headers.version(), 9);
        assert_eq!(headers.file_length(), 849486);
        assert_eq!(headers.dimensions_twips(), (6000, 6000));
        assert_eq!(headers.dimensions(), (300, 300));
        assert_eq!(headers.frame_rate(), 30);
        assert_eq!(headers.frame_count(), 1);
    }

    #[test]
    fn test_902() {
        let (headers, _) = SwfHeaders::open("tests/902.swf").unwrap();
        assert_eq!(headers.signature(), Signature::ZlibCompressed);
        assert_eq!(headers.version(), 9);
        assert_eq!(headers.file_length(), 2032206);
        assert_eq!(headers.dimensions_twips(), (6000, 6000));
        assert_eq!(headers.dimensions(), (300, 300));
        assert_eq!(headers.frame_rate(), 30);
        assert_eq!(headers.frame_count(), 1);
    }

    #[test]
    fn test_submachine_1() {
        let (headers, _) = SwfHeaders::open("tests/submachine_1.swf").unwrap();
        assert_eq!(headers.signature(), Signature::ZlibCompressed);
        assert_eq!(headers.version(), 9);
        assert_eq!(headers.file_length(), 1781964);
        assert_eq!(headers.dimensions_twips(), (8000, 8500));
        assert_eq!(headers.dimensions(), (400, 425));
        assert_eq!(headers.frame_rate(), 25);
        assert_eq!(headers.frame_count(), 29);
    }

    #[test]
    fn test_colourshift() {
        let (headers, _) = SwfHeaders::open("tests/colourshift.swf").unwrap();
        assert_eq!(headers.signature(), Signature::ZlibCompressed);
        assert_eq!(headers.version(), 9);
        assert_eq!(headers.file_length(), 189029);
        assert_eq!(headers.dimensions_twips(), (12800, 9600));
        assert_eq!(headers.dimensions(), (640, 480));
        assert_eq!(headers.frame_rate(), 30);
        assert_eq!(headers.frame_count(), 1);
    }
}
