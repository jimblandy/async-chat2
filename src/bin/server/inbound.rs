//! A task that processes requests from a client.

use async_chat::utils::{self, ChatResult};
use async_chat::Request;
use async_std::net;
use async_std::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::group;
use crate::manager;
use crate::outbound;

/// Handle a single client's connection.
pub async fn serve_connection(
    socket: net::TcpStream,
    manager: manager::CommandQueue,
) -> ChatResult<()> {
    // Start an outbound task.
    let outbound = outbound::new(socket.clone());

    // A table of the chat groups we're in.
    let mut joined: HashMap<Arc<String>, group::CommandQueue> = HashMap::new();

    // Process `Request` values from the client.
    let mut from_client = utils::receive_as_json(socket);
    while let Some(request) = from_client.next().await {
        let request = request?;
        match request {
            // Send a message to a group we have joined.
            Request::Post { group, message } => {
                if let Some(command_queue) = joined.get(&group) {
                    command_queue.send(group::Command::Post { message }).await;
                } else {
                    let message = format!("Not a member of '{}'", group);
                    outbound.send(outbound::Command::Error { message }).await;
                }
            }

            // Join a new group.
            Request::Join { group: group_name } => {
                // Create a one-shot channel for the group manager's reply.
                let (tx, rx) = oneshot::channel();

                // Ask the manager to add us to the group, and send us back a
                // handle on `tx` that we can use to post messages to the group.
                let command = manager::Command::Join {
                    group_name: group_name.clone(),
                    member: outbound.clone(),
                    return_group: tx,
                };
                manager.send(command).await;

                // The manager replies with a Sender<group::Command>, which we
                // can now use to post to the group.
                let group = rx.await.unwrap();
                joined.insert(group_name, group);
            }
        }
    }

    Ok(())
}
