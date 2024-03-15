use log::info;
use tiger_trade_connector::{make_message, read_message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:7496").await?;

    // Logging the listener initialization
    info!("Listening on: 127.0.0.1:7496");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Logging the request content
                info!("received: {:?}", std::str::from_utf8(&buf[0..n]));

                match std::str::from_utf8(&buf[0..n]).unwrap() {
                    "API\0\0\0\0\tv100..176" => {
                        if let Err(e) = socket
                            .write_all(&make_message("176\x0020240209 22:23:12 EST\x00"))
                            .await
                        {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return;
                        }
                    }
                    _ => match read_message(&buf[0..n]) {
                        // 71 corresponds to the "START_API" in the client's outgoing message codes, followed by "2" as
                        // a hardcoded API version, followed by "0" as the client_id.
                        // Client expects to get account id and next valid order id
                        msg if msg.starts_with("71") => {
                            // Next valid order id contains 3 fields:
                            // "9" corresponds to NEXT_VALID_ID in the client's incoming message codes
                            // followed by "1" which represents the field type of message (1 is INT)
                            // followed by "1" which is the value
                            if let Err(e) = socket.write_all(&make_message("9\x001\x001\x00")).await
                            {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                            // Account id contains 3 fields as well:
                            // "15" corresponds to MANAGED_ACCTS
                            // "1" corresponds to INT field type (not sure why, I need to check)
                            // "U12345678" is an account name
                            if let Err(e) = socket
                                .write_all(&make_message("15\x001\x00U12345678\x00"))
                                .await
                            {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                        }
                        _ => {
                            if let Err(e) = socket.write_all(&make_message("unknown\x00\x00")).await
                            {
                                eprintln!("failed to write to socket; err = {:?}", e);
                                return;
                            }
                        }
                    },
                }
                //     _ => {
                //         println!("message: {}", read_message(&buf[0..n]));
                //         if let Err(e) = socket.write_all(&make_message("unknown\0\0")).await {
                //             eprintln!("failed to write to socket; err = {:?}", e);
                //             return;
                //         }
                //     }
                // }
            }
        });
    }
}
