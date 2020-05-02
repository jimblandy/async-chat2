use serde::{Serialize, Deserialize};

/// A request from a chat client.
#[derive(Debug, Deserialize, Serialize)]
pub enum Request {
    /// Please forward messages sent to the given `channel` to this client.
    Subscribe { channel: String },

    /// Please send `message` to `channel`.
    Send { channel: String, message: String },
}

/// A reply from the server to a specific client.
#[derive(Debug, Deserialize, Serialize)]
pub enum Reply {
    /// The given `message` was sent to `channel`, to which this
    /// client is subscribed.
    Message { channel: String, message: String },

    /// We were forced to drop some messages because the queue was full.
    Dropped { count: usize },
}
