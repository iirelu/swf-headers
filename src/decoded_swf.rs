use std::io;
use std::io::Read;
use std::fs::File;

use flate2::FlateReadExt;
use flate2::read::ZlibDecoder;
use lzma;

use super::Signature;
use error::Error;

enum Inner<R: Read> {
    Raw(File),
    Zlib(ZlibDecoder<R>),
    Lzma(lzma::Reader<R>)
}

/// Handles decompressing swf innards and reading the results.
pub struct DecodedSwf {
    _inner: Inner<File>
}

impl DecodedSwf {
    /// Takes a file and a swf signature and returns an EncodedSwf that handles all the
    /// compression for you.
    pub fn decompress(file: File, sig: Signature) -> Result<Self, super::Error> {
        let inner = match sig {
            Signature::Uncompressed => Inner::Raw(file),
            Signature::ZlibCompressed => Inner::Zlib(file.zlib_decode()),
            Signature::LzmaCompressed => Inner::Lzma(try!(lzma::Reader::from(file)))
        };
        Ok(DecodedSwf {
            _inner: inner
        })
    }
}

impl Read for DecodedSwf {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self._inner {
            Inner::Raw(ref mut f) => f.read(buf),
            Inner::Zlib(ref mut f) => f.read(buf),
            Inner::Lzma(ref mut f) => f.read(buf)
        }
    }
}
