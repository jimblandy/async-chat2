//! Utilities for both clients and servers.

use async_std::prelude::*;
use async_std::net;
use serde::Serialize;

/// Our standard `Result` type, with a fully general `Error`.
pub type ChatResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Given a value that can be serialized, transmit it on `socket`.
pub async fn send_as_json<V: Serialize>(socket: &mut net::TcpStream, value: &V) -> ChatResult<()> {
    // Serialize `value` as JSON.
    let mut json = serde_json::to_string(&value)?;
    json.push('\n');
    socket.write_all(json.as_bytes()).await?;
    Ok(())
}

/// Pass `socket` by value to `body`, and log the error `body` returns, if any.
///
/// Taking `socket` as a parameter, instead of just letting `body` capture it
/// itself, gives this function a chance to grab the socket's remote address
/// before passing it along, so we can use it in a later error message.
pub async fn log_socket_error<F, Fut>(socket: net::TcpStream, body: F)
    where F: FnOnce(net::TcpStream) -> Fut,
          Fut: Future<Output=ChatResult<()>>
{
    use std::borrow::Cow;

    let addr_result = socket.peer_addr();
    if let Err(error) = body(socket).await {
        // Convert the address to printable form, if we were able to get it.
        let printable_addr = match addr_result {
            Ok(addr) => Cow::from(addr.to_string()),
            Err(_) => Cow::from("<unknown remote peer>"),
        };

        eprintln!("Error for client {}: {}", printable_addr, error);
    }
}
