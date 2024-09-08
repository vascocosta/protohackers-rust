use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{io, process};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

const ADDRESS: &str = "0.0.0.0:1981";

#[derive(Debug, Deserialize)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Debug, Serialize)]
struct Response {
    method: String,
    prime: bool,
}

fn is_prime(number: f64) -> bool {
    if number <= 1.0 {
        return false;
    }
    if number == 2.0 {
        return true;
    }
    if number % 2.0 == 0.0 {
        return false;
    }

    let max_divider = (number.sqrt()).ceil() as i64;
    for n in 3..max_divider {
        if number as i64 % n == 0 {
            return false;
        }
    }

    true
}

fn handle_request(request: Request) -> Result<Response, &'static str> {
    if request.method != "isPrime" {
        Err("Unknown method.")
    } else {
        Ok(Response {
            method: String::from("isPrime"),
            prime: is_prime(request.number),
        })
    }
}

async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) -> io::Result<()> {
    // Split  the socket into read/write halves for concurrent operations.
    // Wrap the read half with a buffered reader for easy line-based reads.
    let (reader, mut writer) = socket.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Loop that reads from the socket line by line and parses requests.
    loop {
        let bytes_read = reader.read_line(&mut line).await.unwrap_or(0);

        // If bytes_read is 0, then the other side disconnected and we break out of the loop.
        if bytes_read == 0 {
            break;
        }

        // Try to parse each line received from the socket into a Request using JSON.
        // If the Request is valid, then try to handle it with handle_request().
        // Otherwise write to the socket an invalidRequest Response and break out of the loop.
        match serde_json::from_str::<Request>(&line) {
            Ok(request) => {
                println!("{:?}", request);
                let response = match handle_request(request) {
                    Ok(response) => response,
                    Err(_) => Response {
                        method: String::from("invalidRequest"),
                        prime: false,
                    },
                };
                println!("{:?}", response);
                writer
                    .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
                    .await?;
                if response.method == "invalidRequest" {
                    break;
                }
            }
            Err(_) => {
                let response = Response {
                    method: String::from("invalidRequest"),
                    prime: false,
                };
                println!("{:?}", response);
                writer
                    .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
                    .await?;
                break;
            }
        };

        line.clear();
    }

    // When we reach here, it means we broke out of the loop to end the connection.
    // Thus we call shutdown() on the socket and print a disconnect message.
    socket.shutdown().await?;
    println!("Disconnect from: {}", addr);

    Ok(())
}

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind(ADDRESS).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("Error binding to address: {}", error);
            process::exit(1);
        }
    };

    // Loop that accepts new connections and spawns a new task for each.
    loop {
        let (socket, addr) = match listener.accept().await {
            Ok((socket, addr)) => {
                println!("Connection from: {}", addr);
                (socket, addr)
            }
            Err(error) => {
                eprintln!("Error accepting connection: {}", error);
                continue;
            }
        };

        // Spawn a new task to run handle_connection() for each new connection.
        task::spawn(async move {
            if let Err(error) = handle_connection(socket, addr).await {
                eprintln!("Error handling connection: {}", error);
            }
        });
    }
}
