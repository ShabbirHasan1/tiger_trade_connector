mod api;
pub use api::Api;

mod next_valid_order_id;
pub use next_valid_order_id::NextValidOrderId;

mod req_account_summary;
pub use req_account_summary::ReqAccountSummary;


// mod publish;
// pub use publish::Publish;

// mod set;
// pub use set::Set;

// mod subscribe;
// pub use subscribe::{Subscribe, Unsubscribe};

// mod ping;
// pub use ping::Ping;

mod unknown;
use tracing::info;
pub use unknown::Unknown;

use crate::{Connection, Frame, Parse, ParseError, Shutdown};

/// Enumeration of supported Redis commands.
///
/// Methods called on `Command` are delegated to the command implementation.
#[derive(Debug)]
pub enum Command {
    Api(Api),
    NextValidOrderId(NextValidOrderId),
    ReqAccountSummary(ReqAccountSummary),
    // Get(Get),
    // Publish(Publish),
    // Set(Set),
    // Subscribe(Subscribe),
    // Unsubscribe(Unsubscribe),
    // Ping(Ping),
    Unknown(Unknown),
}

impl Command {
    /// Parse a command from a received frame.
    ///
    /// The `Frame` must represent a Redis command supported by `mini-redis` and
    /// be the array variant.
    ///
    /// # Returns
    ///
    /// On success, the command value is returned, otherwise, `Err` is returned.
    pub fn from_frame(frame: Frame) -> crate::Result<Command> {
        // The frame value is decorated with `Parse`. `Parse` provides a
        // "cursor" like API which makes parsing the command easier.
        //
        // The frame value must be an array variant. Any other frame variants
        // result in an error being returned.

        info!("in Command::from_frame");

        let mut parse = Parse::new(frame)?;

        info!("parse is: {:?}", parse);

        // All redis commands begin with the command name as a string. The name
        // is read and converted to lower cases in order to do case sensitive
        // matching.
        let command_name = parse.next_string()?.to_lowercase();

        info!("command_name is: {:?}", command_name);

        // Match the command name, delegating the rest of the parsing to the
        // specific command.
        let command = match &command_name[..] {
            "api" => Command::Api(Api::parse_frames(&mut parse)?),
            "71" => Command::NextValidOrderId(NextValidOrderId::parse_frames(&mut parse)?),
            "62" => Command::ReqAccountSummary(ReqAccountSummary::parse_frames(&mut parse)?), // b"9\08\0215\00\0IBM\0STK\0\00.0\0\0\0SMART\0\0USD\0\0\00\0\0\0\0"
            // "get" => Command::Get(Get::parse_frames(&mut parse)?),
            // "publish" => Command::Publish(Publish::parse_frames(&mut parse)?),
            // "set" => Command::Set(Set::parse_frames(&mut parse)?),
            // "subscribe" => Command::Subscribe(Subscribe::parse_frames(&mut parse)?),
            // "unsubscribe" => Command::Unsubscribe(Unsubscribe::parse_frames(&mut parse)?),
            // "ping" => Command::Ping(Ping::parse_frames(&mut parse)?),
            _ => {
                // The command is not recognized and an Unknown command is
                // returned.
                //
                // `return` is called here to skip the `finish()` call below. As
                // the command is not recognized, there is most likely
                // unconsumed fields remaining in the `Parse` instance.
                return Ok(Command::Unknown(Unknown::new(command_name)));
            }
        };

        // Check if there is any remaining unconsumed fields in the `Parse`
        // value. If fields remain, this indicates an unexpected frame format
        // and an error is returned.
        //parse.finish()?;

        // The command has been successfully parsed
        Ok(command)
    }

    /// Apply the command to the specified `Db` instance.
    ///
    /// The response is written to `dst`. This is called by the server in order
    /// to execute a received command.
    pub(crate) async fn apply(
        self,
        dst: &mut Connection,
        shutdown: &mut Shutdown,
    ) -> crate::Result<()> {
        use Command::*;

        match self {
            Api(cmd) => cmd.apply(dst).await,
            NextValidOrderId(cmd) => cmd.apply(dst).await,
            ReqAccountSummary(cmd) => cmd.apply(dst).await,
            // Get(cmd) => cmd.apply(db, dst).await,
            // Publish(cmd) => cmd.apply(db, dst).await,
            // Set(cmd) => cmd.apply(db, dst).await,
            // Subscribe(cmd) => cmd.apply(db, dst, shutdown).await,
            // Ping(cmd) => cmd.apply(dst).await,
            Unknown(cmd) => cmd.apply(dst).await,
            // `Unsubscribe` cannot be applied. It may only be received from the
            // context of a `Subscribe` command.
            // Unsubscribe(_) => Err("`Unsubscribe` is unsupported in this context".into()),
        }
    }

    /// Returns the command name
    pub(crate) fn get_name(&self) -> &str {
        match self {
            Command::Api(_) => "api",
            Command::NextValidOrderId(_) => "next_valid_order_id",
            Command::ReqAccountSummary(_) => "req_account_summary",
            // Command::Get(_) => "get",
            // Command::Publish(_) => "pub",
            // Command::Set(_) => "set",
            // Command::Subscribe(_) => "subscribe",
            // Command::Unsubscribe(_) => "unsubscribe",
            // Command::Ping(_) => "ping",
            Command::Unknown(cmd) => cmd.get_name(),
        }
    }
}
