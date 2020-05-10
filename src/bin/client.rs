use async_chat::{Request, Reply, utils};
use async_std::prelude::*;
use async_std::{io, net, task};
use std::error::Error;
use async_std::prelude::FutureExt;

/// Given a string `input`, return `Some((token, rest))`, where `token` is the
/// first run of non-whitespace characters in `input`, and `rest` is the rest of
/// the string. If the string contains no non-whitespace characters, return
/// `None`.
fn next_token(mut input: &str) -> Option<(&str, &str)> {
    input = input.trim_start();

    if input.is_empty() {
        return None;
    }

    match input.find(char::is_whitespace) {
        Some(space) => Some((&input[0..space], &input[space..])),
        None => Some((input, ""))
    }
}

/// Parse a line (presumably read from the standard input) as a `Request`.
fn parse_command(line: &str) -> Option<Request> {
    let (command, rest) = next_token(line)?;
    if command == "send" {
        let (group, rest) = next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(Request::Send {
            group: group.to_string(),
            message
        });
    } else if command == "join" {
        let (group, rest) = next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(Request::Join {
            group: group.to_string()
        });
    } else {
        return None;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
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

async fn handle_replies(from_server: net::TcpStream) -> Result<(), Box<dyn Error>> {
    // Process one line at a time from the server. Each line should contain the
    // JSON serialization of a `Reply`.
    let mut from_server = io::BufReader::new(from_server).lines().fuse();
    while let Some(reply_json) = from_server.next().await {
        let reply_json = reply_json?;
        // Parse the JSON into a `Reply` value.
        let reply: Reply = serde_json::from_str(&reply_json)?;
        match reply {
            Reply::Message { group, message } => {
                println!("#{}: {}", group, message);
            }
            Reply::Dropped { count } => {
                println!("warning: {} messages dropped", count);
            }
        }
    }

    Ok(())
}

async fn send_commands(mut to_server: net::TcpStream) -> Result<(), Box<dyn Error>> {
    let mut lines = io::BufReader::new(io::stdin()).lines().fuse();
    while let Some(line) = lines.next().await {
        let line = line?;
        if let Some(request) = parse_command(&line) {
            utils::send_as_json(&mut to_server, &request).await?;
        } else {
            eprintln!("Unrecognized command: {:?}", line);
        }
    }

    Ok(())
}
