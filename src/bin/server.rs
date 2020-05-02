#![allow(unused_imports, unused_variables, dead_code)]
use async_chat::{Request, Reply};
use async_std::prelude::*;
use async_std::{sync, io, net, task};
use async_std::sync::{Arc, Mutex, Receiver, Sender};
use std::borrow::Cow;
use std::collections::{HashSet, HashMap};

/// A bounded queue of replies to be sent to a chat client.
struct ClientQueue {
    /// The sending side of a channel being read by the client's
    /// socket-writing task.
    sender: sync::Sender<Reply>,
}

/// A chatroom channel - simply a set of queues for subscribed clients.
type Channel = HashSet<ClientQueue>;

/// A table of named channels.
type Channels = HashMap<String, Channel>;

fn main() -> io::Result<()> {
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    let channels = Channels::new();
    //task::block_on(listen_for_connections(
    Ok(())
}

async fn listen_for_connections(channels: &mut Channels, addr: impl net::ToSocketAddrs) -> io::Result<()> {
    // Create a server socket listening on `addr`.
    let listener = net::TcpListener::bind(addr).await?;

    // Wait for incoming connections on our server socket.
    let mut incoming = listener.incoming();
    loop {
        // Incoming never returns `None`, so we can use an infinite loop and
        // just unwrap here.
        let connection = incoming.next().await.unwrap()?;

        // Start an asynchronous task to handle this connection.
        task::spawn(async {
            let peer_addr = connection.peer_addr()
                .map_or(Cow::from("<unknown remote peer>"),
                        |addr| Cow::from(addr.to_string()));
            if let Err(err) = serve_connection(connection).await {
                eprintln!("Error handling connection from {}: {}",
                          peer_addr, err);
            }
        });
    }
}

async fn serve_connection(connection: net::TcpStream) -> io::Result<()> {
    Ok(())
}
