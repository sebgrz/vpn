use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, WebSocketStream};

type Stream = WebSocketStream<TcpStream>;
type StreamConnections = Arc<Mutex<HashMap<String, Arc<Mutex<Stream>>>>>;

#[tokio::main]
async fn main() {
    let connections: HashMap<String, Arc<Mutex<Stream>>> = HashMap::new();
    let connections = Arc::new(Mutex::new(connections));
    let server_port = env::args().nth(1).expect("port parameter is required");
    let addr = format!("0.0.0.0:{}", server_port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().unwrap();
        println!("Peer address: {}", peer);

        tokio::spawn(handle_connection(peer, stream, connections.clone()));
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, connections: StreamConnections) {
    let peer_str = peer.to_string();
    let ws_stream = accept_async(stream).await.unwrap();
    let ws_stream = Arc::new(Mutex::new(ws_stream));
    connections
        .clone()
        .lock()
        .await
        .insert(peer_str.clone(), ws_stream.clone());

    println!("New WebSocket connection: {}", peer);

    while let Some(msg) = ws_stream.lock().await.next().await {
        let msg = msg.unwrap();
        let second_ws_stream = connections.lock().await;
        match second_ws_stream
            .iter()
            .filter(|k| !k.0.eq(peer_str.as_str()))
            .last()
        {
            Some(second_ws_stream) => {
                let second_ws_stream: Arc<Mutex<Stream>> = second_ws_stream.1.clone();
                second_ws_stream.lock().await.send(msg).await.unwrap();
            }
            None => {
                println!("second client no exists");
            }
        }
    }
}
