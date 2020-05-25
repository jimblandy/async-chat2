use async_chat::utils::{self, ChatResult};
use async_chat::{Reply, Request};
use async_std::prelude::*;
use async_std::{io, net, task};
use std::sync::Arc;

fn main() -> ChatResult<()> {
    let address = std::env::args().nth(1).expect("Usage: client ADDRESS");

    task::block_on(async {
        let socket = net::TcpStream::connect(address).await?;

        // Create one task that processes incoming messages.
        let from_server = handle_replies(socket.clone());

        // Create another task that reads commands from standard input and sends
        // them to the server.
        let to_server = send_commands(socket);

        // Wait for either of the two tasks to finish.
        from_server.race(to_server).await?;

        Ok(())
    })
}

async fn handle_replies(from_server: net::TcpStream) -> ChatResult<()> {
    // Process `Reply` values from the server.
    let mut from_server = utils::receive_as_json::<Reply>(from_server);
    while let Some(reply) = from_server.next().await {
        let reply = reply?;
        match reply {
            Reply::Message { group, message } => {
                println!("message posted to {}: {}", group, message);
            }
            Reply::Error { message } => {
                println!("error from server: {}", message);
            }
        }
    }

    Ok(())
}

async fn send_commands(mut to_server: net::TcpStream) -> ChatResult<()> {
    let mut lines = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next().await {
        let line = line?;
        let request = match parse_command(&line) {
            Some(request) => request,
            None => continue,
        };

        utils::send_as_json(&mut to_server, &request).await?;
    }

    Ok(())
}

/// Parse a line (presumably read from the standard input) as a `Request`.
fn parse_command(line: &str) -> Option<Request> {
    let (command, rest) = get_next_token(line)?;
    if command == "post" {
        let (group, rest) = get_next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(Request::Post {
            group: Arc::new(group.to_string()),
            message: Arc::new(message),
        });
    } else if command == "join" {
        let (group, rest) = get_next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(Request::Join {
            group: Arc::new(group.to_string()),
        });
    } else {
        eprintln!("Unrecognized command: {:?}", line);
        return None;
    }
}

/// Given a string `input`, return `Some((token, rest))`, where `token` is the
/// first run of non-whitespace characters in `input`, and `rest` is the rest of
/// the string. If the string contains no non-whitespace characters, return
/// `None`.
fn get_next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, "")),
    }
}
