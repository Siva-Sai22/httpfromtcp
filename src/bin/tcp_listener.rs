use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

// Modify this to accept an interface of the writer rather than the tcp stream
async fn stream_lines<R>(mut stream: R, tx: mpsc::Sender<String>)
where
    R: AsyncRead + Unpin,
{
    let mut buffer = [0u8; 8];
    let mut pending = String::new();

    loop {
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .expect("Failed to read the file contents!");

        if bytes_read == 0 {
            break;
        }

        pending
            .push_str(str::from_utf8(&buffer[..bytes_read]).expect("Failed to convert to String"));

        while let Some(idx) = pending.find('\n') {
            let line = pending[..idx].to_string();
            if tx.send(line).await.is_err() {
                return;
            }
            pending = pending[idx + 1..].to_string();
        }
    }

    if !pending.is_empty() {
        let _ = tx.send(pending).await;
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:42069")
        .await
        .expect("Failed to bind to address");
    println!("Server listening on port 42069");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("New connection from: {}", addr);

                let (tx, mut rx) = mpsc::channel::<String>(100);

                tokio::spawn(async move {
                    stream_lines(stream, tx).await;
                });

                tokio::spawn(async move {
                    while let Some(line) = rx.recv().await {
                        println!("{line}");
                    }
                    println!("Connection {} closed", addr);
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
