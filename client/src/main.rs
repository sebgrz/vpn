use std::{env, sync::Arc};

use etherparse::SlicedPacket;
use futures_util::{SinkExt, StreamExt};
use riptun::Tun;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol};
use url::Url;

#[tokio::main]
async fn main() {
    // ./client http://10.0.0.2:80/ws
    let server_ws_addr = env::args().nth(1).unwrap();
    let server_ws_url = Url::parse(server_ws_addr.as_str()).unwrap();

    // create websocket connection
    let (ws_stream, _) = connect_async(server_ws_url).await.unwrap();
    let shared_ws_stream = Arc::new(Mutex::new(ws_stream));

    // Create tunnel
    let tun = match Tun::new("vpn%d", 1) {
        Ok(tun) => Arc::new(Mutex::new(tun)),
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };
    let recv_tun = tun.clone();
    let recv_tun = recv_tun.lock().await;

    // Lets make sure we print the real name of our new TUN device.
    println!("[INFO] => Created TUN '{}'!", recv_tun.name());

    let recv_ws_stream = shared_ws_stream.clone();

    let ws_tun_sender = tun.clone();
    tokio::spawn(async move {
        while let Some(msg) = recv_ws_stream.clone().lock().await.next().await {
            let msg = msg.unwrap();
            if let protocol::Message::Binary(msg_bytes) = msg {
                // put received ws data into tun
                let _ = ws_tun_sender
                    .lock()
                    .await
                    .send_via(msg_bytes.len(), &msg_bytes);
            }
        }
    });

    // Create a buffer to read packets into, and setup the queue to receive from.
    let mut buffer: [u8; 1500] = [0x00; 1500];
    let queue = 0;

    let send_ws_stream = shared_ws_stream.clone();
    let mut send_ws_stream = send_ws_stream.lock().await;
    // Loop forever reading packets off the queue.
    loop {
        // Receive the next packet from the specified queue.
        let read = match recv_tun.recv_via(queue, &mut buffer) {
            Ok(read) => read,
            Err(err) => {
                println!("[ERROR] => {}", err);
                return;
            }
        };

        let packet = SlicedPacket::from_ip(&buffer[..read]).unwrap();

        // send tun packets to ws server
        let ws_msg = protocol::Message::Binary(buffer[..read].to_vec());
        send_ws_stream.send(ws_msg).await.unwrap();

        // Print out the amount of data received and the bytes read off the queue.
        println!("[INFO] => Received packet data ({}B): {:?}", read, packet);
    }
}
