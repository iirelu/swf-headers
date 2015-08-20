use std::io;

use byteorder;
use lzma;

/// The error type used by swf-headers.
#[derive(Debug)]
pub enum Error {
    /// Any IO error, either from directly reading files or from other libraries.
    IoError(io::Error),
    /// All-encompassing variant for anything that can't be a swf file.
    NotSwf
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Self {
        use byteorder::Error::*;
        match err {
            UnexpectedEOF => Error::NotSwf,
            Io(error) => error.into()
        }
    }
}

impl From<lzma::Error> for Error {
    fn from(err: lzma::Error) -> Self {
        use lzma::Error::*;
        match err {
            IO(error) => error.into(),
            ByteOrder(error) => error.into(),
            _ => Error::NotSwf
        }
    }
}
