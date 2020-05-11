//! Utilities for both clients and servers.

use async_std::net;
use async_std::prelude::*;
use serde::Serialize;
use std::error::Error;

/// Our standard `Result` type, with a fully general `Error`.
pub type ChatResult<T> = Result<T, Box<dyn Error>>;

/// Given a value that can be serialized, transmit it on `socket`.
pub async fn send_as_json<V: Serialize>(socket: &mut net::TcpStream, value: &V) -> ChatResult<()> {
    // Serialize `value` as JSON.
    let mut json = serde_json::to_string(&value)?;
    json.push('\n');
    socket.write_all(json.as_bytes()).await?;
    Ok(())
}

/// Await `future`, and log any error it returns.
pub async fn log_error<F>(future: F)
where
    F: Future<Output = ChatResult<()>>,
{
    if let Err(err) = future.await {
        eprintln!("Error: {}", err);
    } else {
        eprintln!("exiting successfully");
    }
}
