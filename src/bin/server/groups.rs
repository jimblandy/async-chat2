//! A chat group.

use async_std::sync::{Arc, Weak};
use std::collections::HashMap;
use std::sync::Mutex as SyncMutex;

use crate::outbound::OutboundQueue;

/// A chat group.
pub struct Group {
    /// The name of this group.
    name: Arc<String>,

    /// Arc weak references to all clients that have joined this group. The
    /// order of the elements is not important.
    members: SyncMutex<Vec<Weak<OutboundQueue>>>,
}

impl Group {
    /// Create a new, empty chat group whose name is `name`.
    pub fn new(name: Arc<String>) -> Group {
        Group {
            name,
            members: SyncMutex::new(Vec::new()),
        }
    }

    /// Add `member` to this group.
    pub fn join(&self, member: &Arc<OutboundQueue>) {
        // Get exclusive access to the member list.
        let mut members = self.members.lock().unwrap();

        // The group holds only weak references to its members, so
        // we need to downgrade the `Arc` to a `Weak` here.
        members.push(Arc::downgrade(&member));
    }

    /// Enqueue `message` to be sent to each member of this group.
    pub fn send(&self, message: String) {
        let message = Arc::new(message);

        // Get exclusive access to the member list.
        let mut members = self.members.lock().unwrap();

        // Enqueue the message to all group members.
        // If a send fails, drop the member.
        members.retain(|member| {
            // Try to upgrade the `Weak` pointer to an `Arc`.
            // If this returns `None`, the member was dropped.
            if let Some(member) = member.upgrade() {
                // Since `name` and `message` are `Arc`s, these `clone` calls just
                // increment the reference count. No text is copied.
                let result = member.send(self.name.clone(), message.clone());

                // If all went well, keep this element of `self.members`.
                result.is_ok()
            } else {
                // We couldn't upgrade the `Weak` to an `Arc`, so the
                // `OutboundQueue` has been dropped. Drop this element from
                // `self.members`.
                false
            }
        });
    }
}

/// A table of named chat groups.
pub struct Groups {
    /// A map from names to chat groups.
    table: SyncMutex<HashMap<Arc<String>, Arc<Group>>>,
}

impl Groups {
    /// Construct a new, empty chat group table.
    pub fn new() -> Groups {
        Groups {
            table: SyncMutex::new(HashMap::new())
        }
    }

    /// Return the chat group named `name`, creating it if one does not exist.
    pub fn get_or_create(&self, name: Arc<String>) -> Arc<Group> {
        let mut table = self.table.lock().unwrap();
        table.entry(name.clone()).or_insert_with(move || Arc::new(Group::new(name))).clone()
    }
}
