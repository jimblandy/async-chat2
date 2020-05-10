//! Conveying messages from a group out to individual members.

use async_chat::{Reply, utils};
use async_chat::utils::ChatResult;
use async_std::{net, sync, task};
use async_std::sync::Arc;
use std::sync::Mutex as SyncMutex;

/// The enqueuing end of a queue of messages bound for a specific client.
pub struct OutboundQueue(SyncMutex<InnerQueue>);

/// The guts of `OutboundQueue`, protected by a synchronous mutex.
struct InnerQueue {
    /// A fixed-capacity queue of messages to be sent to the client.
    enqueue: sync::Sender<EnqueuedMessage>,

    /// How many messages we've had to drop since the last time we were able to
    /// enqueue a message.
    dropped: usize
}

/// A message queued for transmission.
#[derive(Debug)]
struct EnqueuedMessage {
    /// The group to which this message was sent.
    group: Arc<String>,

    /// The message contents.
    message: Arc<String>,

    /// The number of messages we had to drop (zero, we hope) before we were
    /// able to enqueue this message.
    dropped: usize,
}

/// Error type returned when we try to send a message to a client that has
/// disconnected.
pub struct DisconnectedError;

impl OutboundQueue {
    /// Create an `OutboundQueue` sending messages on the socket `to_client`.
    pub fn new(to_client: net::TcpStream) -> OutboundQueue {
        // A queue that holds up to ten messages waiting to be transmitted.
        let (enqueue, dequeue) = sync::channel(10);

        // Start a task that dequeues messages and transmits them on the socket.
        task::spawn({
            utils::log_socket_error(to_client, move |to_client| {
                process_send_queue(dequeue, to_client)
            })
        });

        OutboundQueue(SyncMutex::new(InnerQueue {
            enqueue,
            dropped: 0,
        }))
    }

    /// Enqueue `message` to be sent to the client.
    ///
    /// An individual client must not be able to cause trouble for an entire
    /// chat group, so this method does not convey backpressure to its caller:
    /// it always returns quickly, no matter what sort of condition the client
    /// is in. If the client is not able to keep up with the messages enqueued
    /// to it, messages are dropped (and the client is notified, if possible).
    ///
    /// If the network connection has been closed, either on purpose or due to
    /// an error, return `Err(DisconnectedError)`.
    pub fn send(&self, group: Arc<String>, message: Arc<String>) -> Result<(), DisconnectedError> {
        // Get exclusive access to the `InnerQueue` structure. We won't hold the
        // lock for long: `try_send` never blocks. We only need the mutex to
        // keep the dropped count consistent with our success in enqueuing
        // messages.
        let mut inner = self.0.lock().unwrap();

        let message = EnqueuedMessage {
            group,
            message,
            dropped: inner.dropped,
        };

        match inner.enqueue.try_send(message) {
            Ok(()) => {
                // We succeded in enqueuing a message. Reset the dropped count.
                inner.dropped = 0;
                Ok(())
            }
            Err(sync::TrySendError::Full(_)) => {
                // We had to drop a message. Note this to report in our next
                // successful send.
                inner.dropped += 1;
                Ok(())
            }
            Err(sync::TrySendError::Disconnected(_)) => {
                // The connection has closed. The chat group should remove this
                // member.
                Err(DisconnectedError)
            }
        }
    }
}

/// Take messages from `dequeue` and transmit them on `to_client`.
async fn process_send_queue(dequeue: sync::Receiver<EnqueuedMessage>,
                            mut to_client: net::TcpStream)
                            -> ChatResult<()>
{
    while let Ok(m) = dequeue.recv().await {
        if m.dropped > 0 {
            utils::send_as_json(&mut to_client, &Reply::Dropped {
                count: m.dropped
            }).await?
        }

        utils::send_as_json(&mut to_client, &Reply::Message {
            group: m.group,
            message: m.message,
        }).await?;
    }
    Ok(())
}
