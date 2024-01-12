use std::{env, net::SocketAddr};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

#[tokio::main]
async fn main() {
    let server_port = env::args().nth(1).unwrap();
    let addr = format!("0.0.0.0:{}", server_port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().unwrap();
        println!("Peer address: {}", peer);

        tokio::spawn(handle_connection(peer, stream));
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) {
    let mut ws_stream = accept_async(stream).await.unwrap();

    println!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await.unwrap();
        }
    }
}
