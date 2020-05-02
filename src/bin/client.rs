use async_chat::{Request, Reply};
use async_std::prelude::*;
use async_std::{io, net, task};
use std::error::Error;

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
        let (channel, rest) = next_token(rest)?;
        let message = rest.trim_start().to_string();
        return Some(Request::Send {
            channel: channel.to_string(),
            message
        });
    } else if command == "subscribe" {
        let (channel, rest) = next_token(rest)?;
        if !rest.trim_start().is_empty() {
            return None;
        }
        return Some(Request::Subscribe {
            channel: channel.to_string()
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
        let incoming = print_incoming(socket.clone());

        // Create another task that reads commands from standard input and sends
        // them to the server.
        let outgoing = send_commands(socket);

        // Wait for either of the two tasks to finish.
        incoming.race(outgoing).await?;

        Ok(())
    })
}

async fn print_incoming(incoming: net::TcpStream) -> Result<(), Box<dyn Error>> {
    // Process one line at a time from the server. Each line should contain the
    // JSON serialization of a `Reply`.
    let mut incoming = io::BufReader::new(incoming).lines();
    while let Some(reply_json) = incoming.next().await {
        let reply_json = reply_json?;
        // Parse the JSON into a `Reply` value.
        let reply: Reply = serde_json::from_str(&reply_json)?;
        match reply {
            Reply::Message { channel, message } => {
                println!("#{}: {}", channel, message);
            }
            Reply::Dropped { count } => {
                println!("warning: {} messages dropped", count);
            }
        }
    }

    Ok(())
}

async fn send_commands(mut outgoing: net::TcpStream) -> Result<(), Box<dyn Error>> {
    let mut lines = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = lines.next().await {
        let line = line?;
        if let Some(request) = parse_command(&line) {
            // Serialize `request` as JSON.
            let mut request_json = serde_json::to_string(&request)?;
            request_json.push_str("\n");
            outgoing.write_all(request_json.as_bytes()).await?;
        } else {
            eprintln!("Unrecognized command: {:?}", line);
        }
    }

    Ok(())
}
