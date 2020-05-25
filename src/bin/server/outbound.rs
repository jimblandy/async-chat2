//! A task that transmits replies to a client.

use async_chat::utils::{self, ChatResult};
use async_chat::Reply;
use async_std::{net, sync, task};
use std::sync::Arc;

/// Commands for an outbound task. There's only one.
#[derive(Debug)]
pub enum Command {
    /// The given `message` was sent to the chat group named `group`.
    Send {
        group: Arc<String>,
        message: Arc<String>,
    },

    /// Send the given error message to the client.
    Error { message: String },
}

pub type CommandQueue = sync::Sender<Command>;

/// Start a task sending chat replies on the socket `to_client`.
///
/// The task returns a channel on which the caller can transmit commands.
pub fn new(to_client: net::TcpStream) -> CommandQueue {
    // Create the channel on which we'll receive commands.
    let (tx, rx) = sync::channel(1);

    task::spawn(utils::log_error(handle_commands(rx, to_client)));

    tx
}

/// Handle commands received from `rx`, transmitting on `to_client`.
async fn handle_commands(
    rx: sync::Receiver<Command>,
    mut to_client: net::TcpStream,
) -> ChatResult<()> {
    while let Ok(command) = rx.recv().await {
        match command {
            Command::Send { group, message } => {
                let reply = Reply::Message { group, message };
                utils::send_as_json(&mut to_client, &reply).await?;
            }

            Command::Error { message } => {
                let reply = Reply::Error { message };
                utils::send_as_json(&mut to_client, &reply).await?;
            }
        }
    }

    Ok(())
}
