//! A chat group.

use async_chat::utils::{self, ChatResult};
use async_std::{sync, task};
use std::sync::Arc;

use crate::outbound;

/// Commands understood by a chat group.
pub enum Command {
    /// Add `member` to the group.
    AddMember { member: outbound::CommandQueue },

    /// Post `message` to the group.
    Post { message: Arc<String> },
}

pub type CommandQueue = sync::Sender<Command>;

/// Create a new chat group named `name`.
///
/// Return a command queue we can use to communicate with this group.
pub fn new(name: Arc<String>) -> CommandQueue {
    let (tx, rx) = sync::channel(1);

    task::spawn(utils::log_error(handle_commands(rx, name)));

    tx
}

async fn handle_commands(
    rx: sync::Receiver<Command>,
    group_name: Arc<String>,
) -> ChatResult<()> {
    let mut members = Vec::new();

    while let Ok(command) = rx.recv().await {
        match command {
            Command::AddMember { member } => {
                members.push(member);
            }

            Command::Post { message } => {
                // Send message to all members. If a member's queue is full,
                // drop the message. If it is disconnected, remove the member
                // entirely.
                members.retain(|member| {
                    let result = member.try_send(outbound::Command::Send {
                        group: group_name.clone(),
                        message: message.clone(),
                    });

                    match result {
                        // Message enqueued successfully.
                        Ok(()) => true,

                        // Queue was full. Drop message for that client.
                        Err(sync::TrySendError::Full(_)) => true,

                        // Client has exited. Remove client from members.
                        Err(sync::TrySendError::Disconnected(_)) => false,
                    }
                });
            }
        }
    }

    Ok(())
}
