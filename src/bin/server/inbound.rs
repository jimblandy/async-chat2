//! A task that processes requests from a client.

use async_chat::{Request, utils};
use async_std::prelude::*;
use async_std::net;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::group;
use crate::manager;
use crate::outbound;
use crate::utils::ChatResult;

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
            Request::Post {
                group: group_name,
                message,
            } => {
                if let Some(group) = joined.get(&group_name) {
                    group.send(group::Command::Post { message }).await;
                } else {
                    outbound
                        .send(outbound::Command::Error {
                            message: format!("Not a member of '{}'", group_name),
                        })
                        .await;
                }
            }

            // Join a new group.
            Request::Join { group: group_name } => {
                // Create a one-shot channel for the group manager's reply.
                let (tx, rx) = oneshot::channel();

                // Ask the manager to add us to the group, and send us back a
                // handle on `tx` that we can use to post messages to the group.
                manager
                    .send(manager::Command::Join {
                        group: group_name.clone(),
                        member: outbound.clone(),
                        return_group: tx,
                    })
                    .await;

                // The manager sends us a Sender<group::Command> which we can
                // now use to post to the group.
                let group = rx.await.unwrap();
                joined.insert(group_name, group);
            }
        }
    }

    Ok(())
}
