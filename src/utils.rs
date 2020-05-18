//! Utilities for both clients and servers.

use async_std::{io, net};
use async_std::prelude::*;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::error::Error;

/// Our crate's `Error` and `Result` type, designed for flexibility.
pub type ChatError = Box<dyn Error + Send + Sync + 'static>;
pub type ChatResult<T> = Result<T, ChatError>;

/// Given a value that can be serialized, transmit it on `socket`.
pub async fn send_as_json<V: Serialize>(socket: &mut net::TcpStream, value: &V) -> ChatResult<()> {
    // Serialize `value` as JSON.
    let mut json = serde_json::to_string(&value)?;
    json.push('\n');
    socket.write_all(json.as_bytes()).await?;
    Ok(())
}

/// Read lines from `stream`, parse them as the JSON-serialized form of `V`
/// values, and return a `Stream` of `ChatResult<V>` values.
pub fn receive_as_json<V: DeserializeOwned>(socket: net::TcpStream) -> impl Stream<Item=ChatResult<V>> {
    let buffered = io::BufReader::new(socket);
    buffered.lines().map(|result| {
        result
            .map_err(|e| ChatError::from(e))
            .and_then(|line| serde_json::from_str::<V>(&line).map_err(Into::into))
    })
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
