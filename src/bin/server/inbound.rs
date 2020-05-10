//! Receiving and processing messages from clients.

use async_chat::Request;
use async_std::prelude::*;
use async_std::{io, net};
use std::sync::Arc;


use crate::groups::Groups;
use crate::outbound::OutboundQueue;
use crate::utils::ChatResult;

pub async fn serve_connection(socket: net::TcpStream, groups: Arc<Groups>) -> ChatResult<()> {
    // Create a queue of messages headed back to the client.
    let outbound_queue = Arc::new(OutboundQueue::new(socket.clone()));

    // Process one line at a time from the server. Each line should contain the
    // JSON serialization of a `Request`.
    let mut from_client = io::BufReader::new(socket).lines();
    while let Some(request_json) = from_client.next().await {
        let request_json = request_json?;
        // Parse the JSON into a `Request` value.
        let request: Request = serde_json::from_str(&request_json)?;
        match request {
            Request::Join { group: group_name } => {
                let group_name = Arc::new(group_name);
                let group = groups.get_or_create(group_name);
                group.join(&outbound_queue);
            }
            Request::Send { group: group_name, message} => {
                let group_name = Arc::new(group_name);
                let group = groups.get_or_create(group_name);
                group.send(message);
            }
        }
    }

    Ok(())
}
