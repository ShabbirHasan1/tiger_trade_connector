//! Provides a type representing a Redis protocol frame as well as utilities for
//! parsing frames from a byte array.

use bytes::{Buf, Bytes};
use std::convert::TryInto;
use std::fmt;
use std::io::{Cursor, Read};
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use tracing::info;
/// A frame in the Redis protocol.
#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}

impl Frame {
    /// Returns an empty array
    pub(crate) fn array() -> Frame {
        Frame::Array(vec![])
    }

    /// Push a "bulk" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_bulk(&mut self, bytes: Bytes) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Bulk(bytes));
            }
            _ => panic!("not an array frame"),
        }
    }

    /// Push an "integer" frame into the array. `self` must be an Array frame.
    ///
    /// # Panics
    ///
    /// panics if `self` is not an array
    pub(crate) fn push_int(&mut self, value: u64) {
        match self {
            Frame::Array(vec) => {
                vec.push(Frame::Integer(value));
            }
            _ => panic!("not an array frame"),
        }
    }

    /// Checks if an entire message can be decoded from `src`
    /// I need to make some kind of condition or separate function
    /// that checks if first four bytes are b"API\0", or can be read as u32 size
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_four_u8(src)? {
            b"API\0" => {
                info!("Message::check() matched b'API\0'");
                // get_size from get_four_u8(src) to check if size is equal to src.len() - 4,
                // advance if needed, or skip

                // need to offset b"API\0" in order to read size bytes correctly
                src.advance(4);

                let size = get_size(src)? as usize;
                let length = src.get_ref().len() - 8;

                info!("Size is: {}", size);
                info!("Length is: {}", length);

                if size == length {
                    Ok(())
                } else {
                    Err(format!("protocol error; unable to decode an API frame").into())
                }
            }
            actual => {
                // if get_four_u8(src) is equal to src.len() - 4, return Ok(()),
                // else return error below
                if get_size(src)? as usize == src.get_ref().len() - 4 {
                    return Ok(());
                }
                Err(format!("protocol error; invalid frame type byte `{:?}`", actual).into())
            }
        }
    }

    // I need to create a function that reads from the cursor first 4 bytes,
    // if there is not enough bytes, throw incomplete error,
    // if enough bytes then check if those bytes contain b'API0\',
    // if so, then read next 4 bytes, if there is not enough bytes,
    // throw incomplete error, else read message size from those bytes,
    // then read message content (length from the size), if there is
    // not enough bytes, then throw incomplete error, else read those bytes
    // and return a message with type "HANDSHAKE"
    // if first 4 bytes contain something else, we read those 4 bytes
    // and get message size from it, then we return a message with type "REGULAR",
    // else we return incomplete


    // So this check function (instead of get_u8) checks first four bytes content,
    // if they can be decoded as 'API\0' or a valid array of utf-8
    // it returns Ok(), else it returns error. We put it as a match check.

    //Instead of get_line(src) we need to create a function that reads the message
    // content and return it or an error, if message is incomplete

    /// Checks if an entire message can be decoded from `src`
    // pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
    //     match get_u8(src)? {
    //         // b'A' => {
    //         //     info!("Message::check() matched b'A'");
    //         //     get_line_api(src)?;
    //         //     Ok(())
    //         // }
    //         b'+' => {
    //             get_line(src)?;
    //             Ok(())
    //         }
    //         b'-' => {
    //             get_line(src)?;
    //             Ok(())
    //         }
    //         b':' => {
    //             let _ = get_decimal(src)?;
    //             Ok(())
    //         }
    //         b'$' => {
    //             if b'-' == peek_u8(src)? {
    //                 // Skip '-1\r\n'
    //                 skip(src, 4)
    //             } else {
    //                 // Read the bulk string
    //                 let len: usize = get_decimal(src)?.try_into()?;

    //                 // skip that number of bytes + 2 (\r\n).
    //                 skip(src, len + 2)
    //             }
    //         }
    //         b'A' => {
    //             info!("Frame::check() matched b'A'");
    //             if b'-' == peek_u8(src)? {
    //                 info!("Frame::check() matched b'A' and then '-'");
    //                 // Skip '-1\r\n'
    //                 skip(src, 4)
    //             } else {
    //                 info!("Frame::check() matched b'A' and then didn't match '-'");
    //                 // Read the bulk string
    //                 info!("src after check is: {:?}", src);
    //                 // let len: usize = get_decimal_api(src)?.try_into()?;
    //                 // info!("len is: {}", len);

    //                 // skip that number of bytes + 2 (\r\n).
    //                 skip(src, len + 2)
    //             }
    //         }
    //         b'*' => {
    //             let len = get_decimal(src)?;

    //             for _ in 0..len {
    //                 Frame::check(src)?;
    //             }

    //             Ok(())
    //         }
    //         actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into()),
    //     }
    // }

    /// The message has already been validated with `check`.
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_four_u8(src)? {
            b"API\0" => {
                info!("Message::parse() matched b'API\0'");
                // we need to cut the "API" prefix
                 src.advance(4);
                // Read the line and convert it to `Vec<u8>`
                let line = get_line(src)?.to_vec();

                // Convert the line to a String
                let string = String::from_utf8(line)?;

                Ok(Frame::Simple(string))
            }
            _ => unimplemented!(),
        }
    }

    /// The message has already been validated with `check`.
    // pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
    //     match get_u8(src)? {
    //         b'+' => {
    //             // Read the line and convert it to `Vec<u8>`
    //             let line = get_line(src)?.to_vec();

    //             // Convert the line to a String
    //             let string = String::from_utf8(line)?;

    //             Ok(Frame::Simple(string))
    //         }
    //         b'-' => {
    //             // Read the line and convert it to `Vec<u8>`
    //             let line = get_line(src)?.to_vec();

    //             // Convert the line to a String
    //             let string = String::from_utf8(line)?;

    //             Ok(Frame::Error(string))
    //         }
    //         b':' => {
    //             let len = get_decimal(src)?;
    //             Ok(Frame::Integer(len))
    //         }
    //         b'$' => {
    //             if b'-' == peek_u8(src)? {
    //                 let line = get_line(src)?;

    //                 if line != b"-1" {
    //                     return Err("protocol error; invalid frame format".into());
    //                 }

    //                 Ok(Frame::Null)
    //             } else {
    //                 // Read the bulk string
    //                 let len = get_decimal(src)?.try_into()?;
    //                 let n = len + 2;

    //                 if src.remaining() < n {
    //                     return Err(Error::Incomplete);
    //                 }

    //                 let data = Bytes::copy_from_slice(&src.chunk()[..len]);

    //                 // skip that number of bytes + 2 (\r\n).
    //                 skip(src, n)?;

    //                 Ok(Frame::Bulk(data))
    //             }
    //         }
    //         b'*' => {
    //             let len = get_decimal(src)?.try_into()?;
    //             let mut out = Vec::with_capacity(len);

    //             for _ in 0..len {
    //                 out.push(Frame::parse(src)?);
    //             }

    //             Ok(Frame::Array(out))
    //         }
    //         _ => unimplemented!(),
    //     }
    // }

    /// Converts the frame to an "unexpected frame" error
    pub(crate) fn to_error(&self) -> crate::Error {
        format!("unexpected frame: {}", self).into()
    }
}

impl PartialEq<&str> for Frame {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Frame::Simple(s) => s.eq(other),
            Frame::Bulk(s) => s.eq(other),
            _ => false,
        }
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use std::str;

        match self {
            Frame::Simple(response) => response.fmt(fmt),
            Frame::Error(msg) => write!(fmt, "error: {}", msg),
            Frame::Integer(num) => num.fmt(fmt),
            Frame::Bulk(msg) => match str::from_utf8(msg) {
                Ok(string) => string.fmt(fmt),
                Err(_) => write!(fmt, "{:?}", msg),
            },
            Frame::Null => "(nil)".fmt(fmt),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        // use space as the array element display separator
                        write!(fmt, " ")?;
                    }

                    part.fmt(fmt)?;
                }

                Ok(())
            }
        }
    }
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.get_u8())
}

// fn get_four_u8(src: &mut Cursor<&[u8]>) -> Result<[u8; 4], Error> {
//     if src.remaining() < 4 {
//         return Err(Error::Incomplete);
//     }

//     let mut buffer = [0; 4];
//     src.read_exact(&mut buffer).unwrap();

//     // Ok(src.get_u8())
//     Ok(buffer)
// }

fn get_four_u8<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a[u8], Error> {
    if src.remaining() < 4 {
        return Err(Error::Incomplete);
    }

    // let mut buffer = [0; 4];
    // src.read_exact(&mut buffer).unwrap();

    Ok(&src.get_ref()[0..4])
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if src.remaining() < n {
        return Err(Error::Incomplete);
    }

    src.advance(n);
    Ok(())
}

/// Read a new-line terminated decimal
fn get_decimal(src: &mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;

    let line = get_line(src)?;

    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

/// Frame size from the first 4 bytes
fn get_size(src: &mut Cursor<&[u8]>) -> Result<u32, Error> {
    info!("Inside get_size");
    info!("cursor length is {}", src.get_ref().len());
    info!("cursor position is {}", src.position());
    let start = src.position() as usize;
    //this ignores advance and reads from the beginning of the cursor
    let buf = &src.get_ref()[start..start + 4];
    info!("buf is {:?}", buf);
    let size = u32::from_be_bytes(buf.try_into().unwrap());
    Ok(size)
}



/// Read a new-line terminated decimal
// fn get_decimal_api(src: &mut Cursor<&[u8]>) -> Result<u32, Error> {
//     info!("Frame::get_decimal_api()");

//     let mut line = get_line_api(src)?;
//     info!("line is: {:?}", line); // [80, 73] which is second and third byte
//                                   // let decimal = line.get_u32();

//     let decimal = line.get_u32();
//     info!("decimal is: {:?}", decimal);
//     Ok(decimal)
// }

/// Find a line
fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    info!("Inside get_line");
    info!("cursor length is {}", src.get_ref().len());
    info!("cursor position is {}", src.position());
    let size = get_size(src).unwrap();
    info!("size is {}", size);
    // Scan the bytes directly
    let start = src.position() as usize;
    // Scan to the second to last byte
    let end = src.get_ref().len();

    if start + size as usize + 4 == src.get_ref().len() {
        let line = &src.get_ref()[start + 4..end];
        let message = String::from_utf8(line.try_into().unwrap()).unwrap();
        info!("line is {:?}", line);
        info!("message is: {}", message);
        return Ok(line);
    }

    Err(Error::Incomplete)
}


/// Find a line
// fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
//     // Scan the bytes directly
//     let start = src.position() as usize;
//     // Scan to the second to last byte
//     let end = src.get_ref().len() - 1;

//     for i in start..end {
//         if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
//             // We found a line, update the position to be *after* the \n
//             src.set_position((i + 2) as u64);

//             // Return the line
//             return Ok(&src.get_ref()[start..i]);
//         }
//     }

//     Err(Error::Incomplete)
// }

/// Find a line from the API message
// fn get_line_api<'a>(src: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
//     // Scan the bytes directly
//     let start = src.position() as usize;
//     // Scan to the second to last byte
//     let end = src.get_ref().len() - 1;

//     info!(
//         "Iside the get_line_api, start is: {}, end is: {}",
//         start, end
//     );

//     //b"API\x00\x00\x00\x00\tv100..176"; // length

//     for i in start..end {
//         if src.get_ref()[i] == b'\0' {
//             // We found a line, update the position to be *after* the \n
//             src.set_position((i + 1) as u64);

//             // Return the line
//             return Ok(&src.get_ref()[start..i]);
//         }
//     }

//     Err(Error::Incomplete)
// }

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Incomplete => "stream ended early".fmt(fmt),
            Error::Other(err) => err.fmt(fmt),
        }
    }
}
