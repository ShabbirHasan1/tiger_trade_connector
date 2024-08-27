// b"62\01\01\0All\0AccountType,NetLiquidation,TotalCashValue,SettledCash,AccruedCash,BuyingPower,EquityWithLoanValue,PreviousEquityWithLoanValue,GrossPositionValue,ReqTEquity,ReqTMargin,SMA,InitMarginReq,MaintMarginReq,AvailableFunds,ExcessLiquidity,Cushion,FullInitMarginReq,FullMaintMarginReq,FullAvailableFunds,FullExcessLiquidity,LookAheadNextChange,LookAheadInitMarginReq,LookAheadMaintMarginReq,LookAheadAvailableFunds,LookAheadExcessLiquidity,HighestSeverity,DayTradesRemaining,Leverage\0"
// b"63\01\09001\0U12345678\0NetLiquidation\0196.39\0USD\0
use crate::{Connection, Frame, Parse, API_VERSION};

use bytes::Bytes;
use tracing::{debug, info, instrument};

/// Get the value of key.
///
/// If the key does not exist the special value nil is returned. An error is
/// returned if the value stored at key is not a string, because GET only
/// handles string values.
#[derive(Debug)]
pub struct ReqAccountSummary {
    /// Name of the key to get
    version: String,
    req_id: String,
    group: String,
    tags: Vec<String>, // parse value and split using "," as a delimiter
}

impl  ReqAccountSummary {
    /// Create a new `Get` command which fetches `version` and `id`.
    pub fn new(version: impl ToString, req_id: impl ToString, group: impl ToString, tags: impl ToString) -> ReqAccountSummary {
        ReqAccountSummary {
            version: version.to_string(), // originally int
            req_id: req_id.to_string(), // originally int
            group: group.to_string(),
            tags: tags.to_string().split(",").map(|t: &str| t.to_owned()).collect(),
        }
    }

    /// Get the version
    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn req_id(&self) -> &str {
        &self.req_id
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
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
    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<ReqAccountSummary> {
        // The `GET` string has already been consumed. The next value is the
        // name of the key to get. If the next value is not a string or the
        // input is fully consumed, then an error is returned.
        let version = parse.next_string()?;
        let req_id = parse.next_string()?;
        let group = parse.next_string()?;
        let tags = parse.next_string()?.split(",").map(|t: &str| t.to_owned()).collect();

        Ok(ReqAccountSummary { version, req_id, group, tags })
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
        let response = if let Some(value) = self.get_account_summary(&self.req_id) {
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

        // we need to send one more message with account summary end  like b"64\01\09001\0"
        // dst.write_frame(&response).await?;

        Ok(())
    }

    /// New function that returns the next valid order id
    pub(crate) fn get_account_summary(&self, req_id: &str) -> Option<Bytes> {
        info!("Inside get_account_summary");

        info!("req_id is: {}", req_id);
        info!("tags are: {:?}", &self.tags);


        let account_id = "U12345678";
        let tag = "NetLiquidation";
        let value = "196.39";
        let currency = "USD";

        match account_id == "U12345678" {
            true => {
                let value = format!("63\01\0{}\0{}\0{}\0{}\0{}\0", req_id, account_id, tag, value, currency); // b"63\01\09001\0U12345678\0NetLiquidation\0196.39\0USD\0
                Some(Bytes::copy_from_slice(&value.into_bytes()))
            }
            // true => Some(Bytes::copy_from_slice(&API_VERSION.to_string().into_bytes())),
            _ => None,
        }
    }
}
