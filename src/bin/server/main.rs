use async_chat::utils;
use async_chat::utils::ChatResult;
use async_std::prelude::*;
use async_std::{net, task};
use async_std::sync::Arc;

mod groups;
mod inbound;
mod outbound;

use groups::Groups;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: server ADDRESS");
    let groups = Arc::new(Groups::new());
    task::block_on(listen_for_connections(groups, address))?;
    Ok(())
}

async fn listen_for_connections(groups: Arc<Groups>, addr: impl net::ToSocketAddrs) -> ChatResult<()> {
    // Create a server socket listening on `addr`.
    let listener = net::TcpListener::bind(addr).await?;

    // Listen for new connections on our server socket. Listening never returns
    // `None`, so we can use an infinite loop and just unwrap the `Option`.
    let mut new_connections = listener.incoming();
    loop {
        // The `next` method returns:
        // a future of...
        //     an Option of...
        //         a Result of...
        //             a new network connection.
        // So we must `await` the future, `unwrap` the `Option`, and `?` the
        // `Result`. The future handles blocking. The `Result` handles failure.
        let socket = new_connections.next().await.unwrap()?;

        // Start a new task to handle this connection.
        {
            let groups = groups.clone();
            task::spawn(utils::log_socket_error(socket, move |socket| {
                inbound::serve_connection(socket, groups)
            }));
        }
    }
}
