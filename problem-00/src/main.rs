use std::net::SocketAddr;
use std::{io, process};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

const ADDRESS: &str = "0.0.0.0:1981";

async fn handle_connection(mut socket: TcpStream, addr: SocketAddr) -> io::Result<()> {
    // Loop that reads bytes from the socket and writes them back without any change.
    // We read up to 1024 bytes of data into a buffer each time read() is called.
    let mut buf = [0; 1024];
    loop {
        let bytes_read = socket.read(&mut buf).await.unwrap_or_default();

        // If bytes_read is 0, then the other side disconnected and we break out of the loop.
        if bytes_read == 0 {
            break;
        }

        // If we get an error while writing to the socket, we also break out of the loop.
        if let Err(error) = socket.write_all(&buf[..bytes_read]).await {
            eprintln!("Error writing to socket: {}", error);
            break;
        }
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
