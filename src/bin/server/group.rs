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

async fn handle_commands(rx: sync::Receiver<Command>, group_name: Arc<String>) -> ChatResult<()> {
    let mut next_id = 0;
    let mut members = Vec::new();

    while let Ok(command) = rx.recv().await {
        match command {
            Command::AddMember { member } => {
                next_id += 1;
                members.push((member, next_id));
                eprintln!("adding member {}", next_id);
            }

            Command::Post { message } => {
                // Send message to all members. If a member's queue is full,
                // drop the message. If it is disconnected, remove the member
                // entirely.
                members.retain(|(ref member, ref id)| {
                    eprint!("trying to send to {}: ", id);
                    let result = member.try_send(outbound::Command::Message {
                        group: group_name.clone(),
                        message: message.clone(),
                    });

                    match result {
                        // Message enqueued successfully.
                        Ok(()) => {
                            eprintln!("ok");
                            true
                        }

                        // Queue was full. Drop message for that client.
                        Err(sync::TrySendError::Full(_)) => {
                            eprintln!("full");
                            true
                        }

                        // Client has exited. Remove client from members.
                        Err(sync::TrySendError::Disconnected(_)) => {
                            eprintln!("disconnected; dropping");
                            false
                        }
                    }
                });
            }
        }
    }

    Ok(())
}
