//! A task that transmits replies to a client.

use async_chat::utils::ChatResult;
use async_chat::{utils, Reply};
use async_std::sync::Arc;
use async_std::{net, sync, task};

/// Commands for an outbound task. There's only one.
#[derive(Debug)]
pub enum Command {
    /// The given `message` was sent to the chat group named `group`.
    Message {
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

    task::spawn(utils::log_error(async {
        let result = handle_commands(rx, to_client).await;
        eprintln!("outbound quitting");
        result
    }));

    tx
}

/// Handle commands received from `rx`, transmitting on `to_client`.
async fn handle_commands(
    rx: sync::Receiver<Command>,
    mut to_client: net::TcpStream,
) -> ChatResult<()> {
    while let Ok(command) = rx.recv().await {
        eprintln!("outbound got command: {:?}", command);
        match command {
            Command::Message { group, message } => {
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
