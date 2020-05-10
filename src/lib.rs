use serde::{Serialize, Deserialize};
use std::sync::Arc;

pub mod utils;

/// A request from a chat client.
#[derive(Debug, Deserialize, Serialize)]
pub enum Request {
    /// Please forward messages sent to the given `group` to this client.
    Join { group: String },

    /// Please send `message` to `group`.
    Send { group: String, message: String },
}

/// A reply from the server to a specific client.
#[derive(Debug, Deserialize, Serialize)]
pub enum Reply {
    /// The given `message` was sent to `group`, of which this client is a
    /// member.
    ///
    /// As used in the server, the group name and message are often being sent
    /// to every group member simultaneously. Using `Arc` here lets all this
    /// activity share a single copy of the message.
    Message { group: Arc<String>, message: Arc<String> },

    /// We were forced to drop `count` messages because the queue was full. This
    /// reply takes the place of the dropped messages in the reply stream.
    Dropped { count: usize },
}
