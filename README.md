# tiger-trade-connector
(Under developoment...)

A cross-platform middleware that connects (tiger.trade terminal)[https://www.tiger.trade/terminal] to various data sources and brokers.

This tool can mimic the [Trader Workstation](https://www.interactivebrokers.com/en/trading/tws.php) API in order to simplify connectivity from the [tiger.trade](https://www.tiger.trade) terminal, but gets data from one of supported market data feeds and process orders through one of supported brokers.

## Supported Market Data Feeds
* [polygon.io](https://polygon.io/)

## Supported Brokers
* [alpaca.markets](https://alpaca.markets/)




# TODO

1. Find out how to check stream length
2. Find out how to skip read bytes




# TWS API Notes

## Initial handshake

After the socket has been opened, there must be an initial handshake in which information is exchanged about the highest version supported by TWS and the API. This is important because API messages can have different lengths and fields in different versions and it is necessary to have a version number to interpret received messages correctly.

For this reason it is important that the main EReader object is not created until after a connection has been established. The initial connection results in a negotiated common version between TWS and the API client which will be needed by the EReader thread in interpreting subsequent messages.
After the highest version number which can be used for communication is established, TWS will return certain pieces of data that correspond specifically to the logged-in TWS user’s session. This includes (1) the account number(s) accessible in this TWS session, (2) the next valid order identifier (ID), and (3) the time of connection. In the most common mode of operation the EClient.AsyncEConnect field is set to false and the initial handshake is taken to completion immediately after the socket connection is established. TWS will then immediately provides the API client with this information.

Important: The IBApi.EWrapper.nextValidID callback is commonly used to indicate that the connection is completed and other messages can be sent from the API client to TWS. There is the possibility that function calls made prior to this time could be dropped by TWS.

## Recieving a message
1. data is read from socket to buffer by the read_message function
2. fields are read from the buffer using read_fields function (size, text, rest) tuple


## Function that does processing
connection.read_frame -> connection.parse_frame -> Frame::check -> get_u8(match condition) -> get_line

**get_u8** reads first byte from the buffer and return it, but only if there is more bytes in the cursor, if not it will return the incomplete error
**get_line** checks the length of the cursor and reads from the second byte till the end of cursor where it checks for the end of line char "\r"
then it shifts the cursor position on 2 and returns the line (it allows next run to read a new line), returns incomplete error when it finishs the reading
without finding the "\r" char.


## Establish a connection
Here is a simple flow chart:

```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;
```



## Dont convert bytes into string before the parse
