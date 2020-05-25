//! Asynchronous chat server.

use async_chat::utils::{self, ChatResult};
use async_std::prelude::*;
use async_std::{net, task};

mod group;
mod inbound;
mod manager;
mod outbound;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    let manager = manager::new();
    task::block_on(listen_for_connections(address, manager))?;
    Ok(())
}

async fn listen_for_connections(
    addr: impl net::ToSocketAddrs,
    manager: manager::CommandQueue,
) -> ChatResult<()> {
    // Create a server socket listening on `addr`.
    let listener = net::TcpListener::bind(addr).await?;

    // Listen for new connections on our server socket.
    let mut new_connections = listener.incoming();
    loop {
        // The `next` method returns:
        // a future of...
        //     an Option of...
        //         a Result of...
        //             a new network connection.
        // So we must `await` the future, `unwrap` the `Option`,
        // and `?` the `Result`.
        let socket = new_connections.next().await.unwrap()?;

        // Start a new task to handle this connection.
        {
            let manager = manager.clone();
            let future = inbound::serve_connection(socket, manager);
            task::spawn(utils::log_error(future));
        }
    }
}
