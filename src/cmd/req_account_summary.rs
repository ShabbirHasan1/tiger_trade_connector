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
        let mut response = Frame::array();
        let account_type = self.get_tag_value(&self.req_id, "AccountType", "INDIVIDUAL", "");
        let net_liquidation = self.get_tag_value(&self.req_id, "NetLiquidation", "196.35", "USD");
        let total_cash_value = self.get_tag_value(&self.req_id, "TotalCashValue", "51.39", "USD");
        let accrued_cache = self.get_tag_value(&self.req_id, "AccruedCash", "0.00", "USD");
        let buying_power = self.get_tag_value(&self.req_id, "BuyingPower", "51.39", "USD");
        let equity_with_loan_value = self.get_tag_value(&self.req_id, "EquityWithLoanValue", "196.35", "USD");
        let gross_position_value = self.get_tag_value(&self.req_id, "GrossPositionValue", "144.96", "USD");
        let sma = self.get_tag_value(&self.req_id, "SMA", "125.84", "USD");
        let init_margin_req = self.get_tag_value(&self.req_id, "InitMarginReq", "145.01", "USD");
        let maint_margin_req = self.get_tag_value(&self.req_id, "MaintMarginReq", "50.74", "USD");
        let available_funds = self.get_tag_value(&self.req_id, "AvailableFunds", "51.39", "USD");
        let excess_liquidity = self.get_tag_value(&self.req_id, "ExcessLiquidity", "145.61", "USD");
        let cushion = self.get_tag_value(&self.req_id, "Cushion", "0.741604", "");
        let full_init_margin_req = self.get_tag_value(&self.req_id, "FullInitMarginReq", "145.01", "USD");
        let full_maint_margin_req = self.get_tag_value(&self.req_id, "FullMaintMarginReq", "50.75", "USD");
        let full_available_funds = self.get_tag_value(&self.req_id, "FullAvailableFunds", "51.39", "USD");
        let full_excess_liquidity = self.get_tag_value(&self.req_id, "FullExcessLiquidity", "145.61", "USD");
        let look_ahead_next_change = self.get_tag_value(&self.req_id, "LookAheadNextChange", "1724702400", "");
        let look_ahead_init_margin_req = self.get_tag_value(&self.req_id, "LookAheadInitMarginReq", "144.96", "USD");
        let look_ahead_maint_margin_req = self.get_tag_value(&self.req_id, "LookAheadMaintMarginReq", "50.74", "USD");
        let look_ahead_available_funds = self.get_tag_value(&self.req_id, "LookAheadAvailableFunds", "51.39", "USD");
        let look_ahead_excess_liquidity = self.get_tag_value(&self.req_id, "LookAheadExcessLiquidity", "145.61", "USD");
        let day_trades_remaining = self.get_tag_value(&self.req_id, "DayTradesRemaining", "3", "");
        let leverage = self.get_tag_value(&self.req_id, "Leverage", "0.74", "");
        let end_summary: Bytes = Bytes::copy_from_slice(&format!("64\01\0{}\0", &self.req_id).into_bytes());

        response.push_bulk(account_type); // for each tag we need to send a separate message
        response.push_bulk(net_liquidation);
        response.push_bulk(total_cash_value);
        response.push_bulk(accrued_cache);
        response.push_bulk(buying_power);
        response.push_bulk(equity_with_loan_value);
        response.push_bulk(gross_position_value);
        response.push_bulk(sma);
        response.push_bulk(init_margin_req);
        response.push_bulk(maint_margin_req);
        response.push_bulk(available_funds);
        response.push_bulk(excess_liquidity);
        response.push_bulk(cushion);
        response.push_bulk(full_init_margin_req);
        response.push_bulk(full_maint_margin_req);
        response.push_bulk(full_available_funds);
        response.push_bulk(full_excess_liquidity);
        response.push_bulk(look_ahead_next_change);
        response.push_bulk(look_ahead_init_margin_req);
        response.push_bulk(look_ahead_maint_margin_req);
        response.push_bulk(look_ahead_available_funds);
        response.push_bulk(look_ahead_excess_liquidity);
        response.push_bulk(day_trades_remaining);
        response.push_bulk(leverage);
        response.push_bulk(end_summary); // we need to send one more message with account summary end  like b"64\01\09001\0"

        debug!(?response);

        // Write the response back to the client
        dst.write_frame(&response).await?;

        Ok(())
    }

    /// New function that returns the account summary with all requested tags and values
    pub(crate) fn get_tag_value(&self, req_id: &str, tag: &str, value: &str, currency: &str) -> Bytes {
        info!("Inside get_account_summary");

        info!("req_id is: {}", req_id);
        //info!("tags are: {:?}", &self.tags);


        let account_id = "U12345678";

        let value = format!("63\01\0{}\0{}\0{}\0{}\0{}\0", req_id, account_id, tag, value, currency); // b"63\01\09001\0U12345678\0NetLiquidation\0196.39\0USD\0
        Bytes::copy_from_slice(&value.into_bytes())
    }
}
