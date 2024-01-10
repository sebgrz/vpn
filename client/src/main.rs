use etherparse::SlicedPacket;
use riptun::Tun;

#[tokio::main]
async fn main() {
    let tun = match Tun::new("vpn%d", 1) {
        Ok(tun) => tun,
        Err(err) => {
            println!("[ERROR] => {}", err);
            return;
       }
    };

    // Lets make sure we print the real name of our new TUN device.
    println!("[INFO] => Created TUN '{}'!", tun.name());

    // Create a buffer to read packets into, and setup the queue to receive from.
    let mut buffer: [u8; 1500] = [0x00; 1500];
    let queue = 0;

    // Loop forever reading packets off the queue.
    loop {
        // Receive the next packet from the specified queue.
        let read = match tun.recv_via(queue, &mut buffer) {
            Ok(read) => read,
            Err(err) => {
                println!("[ERROR] => {}", err);
                return;
            }
        };

        let packet = SlicedPacket::from_ip(&buffer[..read]).unwrap();

        // Print out the amount of data received and the bytes read off the queue.
        println!(
            "[INFO] => Received packet data ({}B): {:?}",
            read,
            packet
        );
    }
}
