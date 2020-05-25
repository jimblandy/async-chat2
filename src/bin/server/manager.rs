//! A manager for a list of chat groups.

use async_chat::utils::{self, ChatResult};
use async_std::{sync, task};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::group;
use crate::outbound;

/// Commands understood by a chat group manager.
pub enum Command {
    /// A client wishes to join a chat group.
    Join {
        /// The client that wishes to join. Specifically, a queue by which we
        /// can send it the messages posted to the group.
        member: outbound::CommandQueue,

        /// The name of the chat group to join. If the group does not exist,
        /// create it.
        group_name: Arc<String>,

        /// The client would like to post messages as well, so we should
        /// send it the chat group's command queue on this one-shot.
        return_group: oneshot::Sender<group::CommandQueue>,
    },
}

pub type CommandQueue = sync::Sender<Command>;

/// Create a new chat group manager.
///
/// Return a command queue we can use to communicate with it.
pub fn new() -> CommandQueue {
    let (tx, rx) = sync::channel(1);

    task::spawn(utils::log_error(handle_commands(rx)));

    tx
}

/// Handle commands sent to us on `rx`.
async fn handle_commands(rx: sync::Receiver<Command>) -> ChatResult<()> {
    // A hash table of chat groups.
    let mut groups = HashMap::new();

    while let Ok(command) = rx.recv().await {
        match command {
            Command::Join {
                member,
                group_name,
                return_group,
            } => {
                // Find the group, or create one if it does not exist.
                let group = groups
                    .entry(group_name.clone())
                    .or_insert_with(|| group::new(group_name));

                // Add this client to the group.
                group.send(group::Command::AddMember { member }).await;

                // Send this group's command queue back to the client, so
                // it can post messages.
                let _ = return_group.send(group.clone());
            }
        }
    }

    Ok(())
}
