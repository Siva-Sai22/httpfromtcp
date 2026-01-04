use std::io::{Write, stdout};

use tokio::{
    io::{AsyncBufReadExt, BufReader, stdin},
    net::UdpSocket,
};

#[tokio::main]
async fn main() {
    let sock = UdpSocket::bind("0.0.0.0:0")
        .await
        .expect("Failed to bind UDP socket");
    sock.connect("127.0.0.1:42069")
        .await
        .expect("Failed to connect to remote address");
    println!("UDP sender is running.");

    let stdin = BufReader::new(stdin());
    let mut lines = stdin.lines();

    loop {
        print!("> ");
        stdout().flush().expect("Failed to flush stdout");
        while let Some(line) = lines.next_line().await.expect("Failed to read line") {
            let msg = line + "\n";

            sock.send(msg.as_bytes())
                .await
                .expect("Failed to send message");

            print!("> ");
            stdout().flush().expect("Failed to flush stdout");
        }
    }
}
