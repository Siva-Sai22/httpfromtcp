use tokio::io::AsyncReadExt;
use tokio::{fs::File, sync::mpsc};

async fn stream_lines(mut file: File, tx: mpsc::Sender<String>) {
    let mut buffer = [0u8; 8];
    let mut pending = String::new();

    loop {
        let bytes_read = file
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
    let file = File::open("messages.txt")
        .await
        .expect("Failed to read the file!");

    let (tx, mut rx) = mpsc::channel::<String>(100);
    tokio::spawn(async move {
        stream_lines(file, tx).await;
    });

    while let Some(line) = rx.recv().await {
        println!("read: {line}");
    }
}
