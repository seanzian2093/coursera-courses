use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tungstenite::protocol::Message;

#[tokio::main]
pub async fn main() {
    // let url = url::Url::parse("ws://127.0.0.1:8080").unwrap();
    let url = "ws://127.0.0.1:8080";

    // Establish WebSocket connection
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket connection established");

    // Send a message to the WebSocket server
    let message = Message::Text("Hello, server!".into());
    ws_stream
        .send(message)
        .await
        .expect("Failed to send message");

    // Receive and print the response from the server
    if let Some(Ok(msg)) = ws_stream.next().await {
        println!("Received message from server: {}", msg);
    }
}
