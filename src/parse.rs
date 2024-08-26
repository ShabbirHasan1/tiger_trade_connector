// use crate::{clients::Message, Frame};
use crate::{Frame};
use bytes::{Bytes, BytesMut};
use tracing::info;
use core::str;
use std::{fmt, vec};

/// Utility for parsing a command
///
/// Commands are represented as array frames. Each entry in the frame is a
/// "token". A `Parse` is initialized with the array frame and provides a
/// cursor-like API. Each command struct includes a `parse_frame` method that
/// uses a `Parse` to extract its fields.
#[derive(Debug)]
pub(crate) struct Parse {
    /// Array frame iterator.
    // parts: vec::IntoIter<Frame>,
    fields: vec::IntoIter<Vec<u8>>,
}

/// Error encountered while parsing a frame.
///
/// Only `EndOfStream` errors are handled at runtime. All other errors result in
/// the connection being terminated.
#[derive(Debug)]
pub(crate) enum ParseError {
    /// Attempting to extract a value failed due to the frame being fully
    /// consumed.
    EndOfStream,

    /// All other errors
    Other(crate::Error),
}

impl Parse {
    /// Create a new `Parse` to parse the contents of `frame`.
    ///
    /// Returns `Err` if `frame` is not an array frame.
    pub(crate) fn new(frame: Frame) -> Result<Parse, ParseError> {
        let message: Bytes = match frame {
            Frame::Api(message) => {
                // we need to add b"api" to the beginning of the message
                let prefix = Bytes::from("api\0");
                let mut combined = BytesMut::with_capacity(prefix.len() + message.len());

                combined.extend_from_slice(&prefix);
                combined.extend_from_slice(&message);
                combined.freeze()
            }
            Frame::Bulk(message) => message,
            frame => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
        };

        let fields: Vec<Vec<u8>> = message
            .split(|&b| b == b'\0')
            .map(|slice| slice.to_vec())
            .collect();

        info!("fields are: {:?}", fields);

        Ok(Parse {
            fields: fields.into_iter()
        })
    }


    // pub(crate) fn new(frame: Frame) -> Result<Parse, ParseError> {
    //     let array = match frame {
    //         Frame::Array(array) => array,
    //         frame => return Err(format!("protocol error; expected array, got {:?}", frame).into()),
    //     };

    //     Ok(Parse {
    //         parts: array.into_iter(),
    //     })
    // }


    /// Return the next entry. Array frames are arrays of frames, so the next
    /// entry is a frame.
    fn next(&mut self) -> Result<Vec<u8>, ParseError> {
        self.fields.next().ok_or(ParseError::EndOfStream)
    }

    /// Return the next entry as a string.
    ///
    /// If the next entry cannot be represented as a String, then an error is returned.
    pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {

        let v = self.next()?;
        str::from_utf8(v.as_slice())
            .map(|s| s.to_string())
            .map_err(|_| "protocol error; invalid string".into())
    }


    // pub(crate) fn next_string(&mut self) -> Result<String, ParseError> {
    //     match self.next()? {
    //         // Both `Simple` and `Bulk` representation may be strings. Strings
    //         // are parsed to UTF-8.
    //         //
    //         // While errors are stored as strings, they are considered separate
    //         // types.
    //         Frame::Simple(s) => Ok(s),
    //         Frame::Bulk(data) => str::from_utf8(&data[..])
    //             .map(|s| s.to_string())
    //             .map_err(|_| "protocol error; invalid string".into()),
    //         frame => Err(format!(
    //             "protocol error; expected simple frame or bulk frame, got {:?}",
    //             frame
    //         )
    //         .into()),
    //     }
    // }


    /// Return the next entry as raw bytes.
    ///
    /// If the next entry cannot be represented as raw bytes, an error is
    /// returned.
    // pub(crate) fn next_bytes(&mut self) -> Result<Bytes, ParseError> {
    //     match self.next()? {
    //         // Both `Simple` and `Bulk` representation may be raw bytes.
    //         //
    //         // Although errors are stored as strings and could be represented as
    //         // raw bytes, they are considered separate types.
    //         Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
    //         Frame::Bulk(data) => Ok(data),
    //         frame => Err(format!(
    //             "protocol error; expected simple frame or bulk frame, got {:?}",
    //             frame
    //         )
    //         .into()),
    //     }
    // }

    /// Return the next entry as an integer.
    ///
    /// This includes `Simple`, `Bulk`, and `Integer` frame types. `Simple` and
    /// `Bulk` frame types are parsed.
    ///
    /// If the next entry cannot be represented as an integer, then an error is
    /// returned.
    // pub(crate) fn next_int(&mut self) -> Result<u64, ParseError> {
    //     use atoi::atoi;

    //     const MSG: &str = "protocol error; invalid number";

    //     match self.next()? {
    //         // An integer frame type is already stored as an integer.
    //         Frame::Integer(v) => Ok(v),
    //         // Simple and bulk frames must be parsed as integers. If the parsing
    //         // fails, an error is returned.
    //         Frame::Simple(data) => atoi::<u64>(data.as_bytes()).ok_or_else(|| MSG.into()),
    //         Frame::Bulk(data) => atoi::<u64>(&data).ok_or_else(|| MSG.into()),
    //         frame => Err(format!("protocol error; expected int frame but got {:?}", frame).into()),
    //     }
    // }

    /// Ensure there are no more entries in the array
    pub(crate) fn finish(&mut self) -> Result<(), ParseError> {
        if self.fields.next().is_none() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }
}

impl From<String> for ParseError {
    fn from(src: String) -> ParseError {
        ParseError::Other(src.into())
    }
}

impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        src.to_string().into()
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EndOfStream => "protocol error; unexpected end of stream".fmt(f),
            ParseError::Other(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}
