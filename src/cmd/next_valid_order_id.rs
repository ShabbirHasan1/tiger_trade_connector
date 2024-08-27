use crate::{Connection, Frame, Parse, API_VERSION};

use bytes::Bytes;
use tracing::{debug, info, instrument};

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
#[derive(Debug)]
pub struct NextValidOrderId {
    /// Name of the key to get
    version: String,
    client_id: String,
}

impl  NextValidOrderId {
    /// Create a new `Get` command which fetches `version` and `id`.
    pub fn new(version: impl ToString, client_id: impl ToString) -> NextValidOrderId {
        NextValidOrderId {
            version: version.to_string(),
            client_id: client_id.to_string(),
        }
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
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
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<NextValidOrderId> {
        // The `GET` string has already been consumed. The next value is the
        // name of the key to get. If the next value is not a string or the
        // input is fully consumed, then an error is returned.
        let version = parse.next_string()?;
        let client_id = parse.next_string()?;

        Ok(NextValidOrderId { version, client_id })
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

        let mut response = Frame::array();
        let order_id = self.get_next_valid_order_id(&self.client_id);
        let account_id = self.get_user_account_id(&self.client_id);
        
        response.push_bulk(order_id); // for each tag we need to send a separate message
        response.push_bulk(account_id); // we need to send one more message with account summary end  like b"64\01\09001\0"
 

        debug!(?response);

        // Write the response back to the client
        dst.write_frame(&response).await?;

        Ok(())
    }

    /// New function that returns the next valid order id
    pub(crate) fn get_next_valid_order_id(&self, client_id: &str) -> Bytes {
        info!("Inside get_next_valid_order_id");

        info!("client_id is: {}", client_id);

        let order_id = 1;

        let value = format!("9\01\0{}\0", order_id);
        Bytes::copy_from_slice(&value.into_bytes())
    }

    /// New function that returns the next valid order id
    pub(crate) fn get_user_account_id(&self, client_id: &str) -> Bytes {
        info!("Inside get_user_account_id");

        info!("client_id is: {}", client_id);

        let account_id = "U12345678"; // one or coma separated list

        let value = format!("15\01\0{}\0", account_id);
        Bytes::copy_from_slice(&value.into_bytes())
    }
}
