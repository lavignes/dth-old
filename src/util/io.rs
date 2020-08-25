use crate::util::BoxedError;
use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader, ErrorKind, Read, Seek},
    path::Path,
    str::FromStr,
};

/// Easy way to return something that's error-like wrapped in an `std::io::Error`
#[inline]
pub fn io_err<T, E: Into<BoxedError>>(kind: ErrorKind, err: E) -> io::Result<T> {
    Err(io::Error::new(kind, err))
}

/// Wrap an error in a `std::io::Error` representing invalid data.
pub fn invalid_data<E: Into<BoxedError>>(err: E) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, err)
}

/// Open a file at some path, returning a buffered reader.
///
/// Returns a more helpful error if the file cannot be opened.
pub fn buf_open<P: AsRef<Path>>(path: P) -> io::Result<impl BufRead + Seek> {
    let path: &Path = path.as_ref();
    let file = File::open(path).map_err(|err| {
        io::Error::new(
            ErrorKind::Other,
            format!("Could not open file {}: {}", path.display(), err),
        )
    })?;
    Ok(BufReader::new(file))
}

/// Parse something from a string but fail with a `std::io::Error`
#[inline]
pub fn parse<F: FromStr<Err = E>, E: Into<BoxedError>>(s: &str) -> io::Result<F> {
    s.parse().map_err(invalid_data)
}

/// Parse something from a string but fail with a `std::io::Error` and some sort of helpful message
///
/// Example:
///
/// ```
/// use dth::util;
/// // fails with: "I was expecting an i32: <underlying FromStr error>"
/// let x: i32 = util::parse_diagnostic("q", "I was expecting an i32")?;
/// ```
#[inline]
pub fn parse_diagnostic<F: FromStr<Err = E>, E: Into<BoxedError>, D: Display>(
    s: &str,
    diagnostic: &D,
) -> io::Result<F> {
    s.parse()
        .map_err(|err: F::Err| invalid_data(format!("{}: {}", diagnostic, err.into())))
}

#[inline]
pub fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
    let mut bytes = [0u8; 4];
    if reader.read(&mut bytes)? != bytes.len() {
        io_err(ErrorKind::UnexpectedEof, "Could not read enough bytes")
    } else {
        Ok(u32::from_le_bytes(bytes))
    }
}

#[inline]
pub fn read_u16<R: Read>(reader: &mut R) -> io::Result<u16> {
    let mut bytes = [0u8; 2];
    if reader.read(&mut bytes)? != bytes.len() {
        io_err(ErrorKind::UnexpectedEof, "Could not read enough bytes")
    } else {
        Ok(u16::from_le_bytes(bytes))
    }
}
