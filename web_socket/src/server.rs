use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Create a TCP listener bound to the local address
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to address");

    println!("WebSocket server running on ws://127.0.0.1:8080");

    // Accept incoming TCP connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    // Upgrade the TCP connection to a WebSocket connection
    let ws_stream = accept_async(stream)
        .await
        .expect("Failed to upgrade to WebSocket");
    println!("New WebSocket connection established");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Send and receive WebSocket messages
    tokio::spawn(async move {
        while let Some(Ok(message)) = ws_receiver.next().await {
            println!("Received message: {}", message);
            ws_sender
                .send(message)
                .await
                .expect("Failed to send message");
        }
    });
}
