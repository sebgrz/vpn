use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};

type SharedStream = Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>;
type StreamConnections = Arc<Mutex<HashMap<String, SharedStream>>>;

#[tokio::main]
async fn main() {
    let connections: HashMap<String, SharedStream> = HashMap::new();
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
    let (send_ws_stream, mut recv_ws_stream) = ws_stream.split();
    let shared_send_stream = Arc::new(Mutex::new(send_ws_stream));

    connections
        .clone()
        .lock()
        .await
        .insert(peer_str.clone(), shared_send_stream);

    println!("New WebSocket connection: {}", peer);

    while let Some(msg) = recv_ws_stream.next().await {
        let msg = msg.unwrap();
        let second_ws_stream = connections.lock().await;
        match second_ws_stream
            .iter()
            .filter(|k| !k.0.eq(peer_str.as_str()))
            .last()
        {
            Some(second_ws_stream) => {
                let second_peer_str = second_ws_stream.0.to_string();
                println!("[INFO] => Received packet data {} -> {} ({}B)", peer_str.to_string(), second_peer_str, msg);
                let second_ws_stream: SharedStream = second_ws_stream.1.clone();
                second_ws_stream.lock().await.send(msg).await.unwrap();
            }
            None => {
                println!("second client no exists");
            }
        }
    }
}
