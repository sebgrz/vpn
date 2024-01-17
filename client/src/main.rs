use std::{env, sync::Arc};

use etherparse::SlicedPacket;
use futures_util::{SinkExt, StreamExt};
use riptun::{Tun, TokioTun};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::protocol};
use url::Url;

#[tokio::main]
async fn main() {
    // ./client http://10.0.0.2:80/ws
    let server_ws_addr = env::args().nth(1).expect("server address is required");
    let server_ws_url = Url::parse(server_ws_addr.as_str()).unwrap();

    let queue = 0;

    // create websocket connection
    let (ws_stream, _) = connect_async(server_ws_url).await.unwrap();
    let (mut send_ws_stream, mut recv_ws_stream) = ws_stream.split();

    // Create tunnel
    let tun = match TokioTun::new("vpn%d", 1) {
        Ok(tun) => Arc::new(tun),
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
        }
    };

    // Lets make sure we print the real name of our new TUN device.
    let ws_tun_sender = tun.clone();
    tokio::spawn(async move {
        while let Some(msg) = recv_ws_stream.next().await {
            let msg = msg.unwrap();
            if let protocol::Message::Binary(msg_bytes) = msg {
                // put received ws data into tun
                let packet = SlicedPacket::from_ip(&msg_bytes).unwrap();
                println!("[INFO] => Received ws packet data: {:?}", packet);
                let sent = ws_tun_sender
                    .send_via(queue, &msg_bytes)
                    .await
                    .unwrap();
                println!("[INFO] => sent to vpn tun: {}", sent);
            }
        }
    });

    // Create a buffer to read packets into, and setup the queue to receive from.
    let mut buffer: [u8; 1500] = [0x00; 1500];
    //let recv_tun = tun.clone();
    let recv_tun = tun;
    println!("[INFO] => Created TUN '{}'!", recv_tun.name());

    // Loop forever reading packets off the queue.
    loop {
        // Receive the next packet from the specified queue.
        let read = match recv_tun.recv_via(queue, &mut buffer).await {
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
