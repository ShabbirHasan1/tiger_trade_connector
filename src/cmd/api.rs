use crate::{Connection, Db, Frame, Parse, API_VERSION};

use bytes::Bytes;
use tracing::{debug, info, instrument};

use chrono::prelude::*;

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
#[derive(Debug)]
pub struct Api {
    /// Name of the key to get
    key: String,
}

impl Api {
    /// Create a new `Get` command which fetches `key`.
    pub fn new(key: impl ToString) -> Api {
        Api {
            key: key.to_string(),
        }
    }

    /// Get the key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Parse a `Get` instance from a received frame.
    ///
    /// The `Parse` argument provides a cursor-like API to read fields from the
    /// `Frame`. At this point, the entire frame has already been received from
    /// the socket.
    ///
    /// The `GET` string has already been consumed.
    ///
    /// # Returns
    ///
    /// Returns the `Get` value on success. If the frame is malformed, `Err` is
    /// returned.
    ///
    /// # Format
    ///
    /// Expects an array frame containing two entries.
    ///
    /// ```text
    /// GET key
    /// ```
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Api> {
        // The `GET` string has already been consumed. The next value is the
        // name of the key to get. If the next value is not a string or the
        // input is fully consumed, then an error is returned.
        let key = parse.next_string()?;

        Ok(Api { key })
    }

    /// Apply the `Get` command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        // Here I need to call a function that can remove "v" from the key and then split
        // the rest using ".." as a separator. If api version in the connector is between
        // min and max valuesm then return <version>\0<date time>0\  ex: "176\x0020240209 22:23:12 EST\x00"
        // Get the value from the shared database state
        let response = if let Some(value) = self.get_version(&self.key) {
            // If a value is present, it is written to the client in "bulk"
            // format.
            info!("Inside response bulk");
            Frame::Bulk(value)
        } else {
            info!("Inside response null");
            // If there is no value, `Null` is written.
            Frame::Null
        };

        debug!(?response);

        // Write the response back to the client
        dst.write_frame(&response).await?;

        Ok(())
    }

    /// New function that returns the API version
    pub(crate) fn get_version(&self, key: &str) -> Option<Bytes> {
        info!("Inside get_version");

        let mut values = key.trim_start_matches('v').split("..");

        info!("values is: {:?}", values);

        let min = values.next()?.parse::<u16>().unwrap();
        let max = values.next()?.parse::<u16>().unwrap();
        info!("min is: {}", min);
        info!("max is: {}", max);

        match min <= API_VERSION && API_VERSION <= max {
            true => {
                let now = Local::now();
                let timestamp = now.format("%Y%m%d %H:%M:%S %Z").to_string();
                let value = format!("{}\0{}\0", API_VERSION.to_string(), timestamp);
                Some(Bytes::copy_from_slice(&value.into_bytes()))
            }
            // true => Some(Bytes::copy_from_slice(&API_VERSION.to_string().into_bytes())),
            false => None,
        }
    }
}
