use std::sync::atomic::Ordering::SeqCst;
use std::{
    io::Error,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    listener: TcpListener,
    closed: AtomicBool,
}

impl Server {
    pub fn close(self: Arc<Self>) {
        self.closed.store(true, SeqCst);
    }

    async fn listen(self: Arc<Self>) {
        loop {
            if self.closed.load(SeqCst) {
                break;
            }
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        server.handle(stream).await;
                    });
                }
                Err(_) => break,
            }
        }
    }

    async fn handle(self: Arc<Self>, mut stream: TcpStream) {
        let data = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\nHello World!";
        if let Err(e) = stream.write_all(data).await {
            eprintln!("Failed to write to stream: {}", e);
        }
        if let Err(e) = stream.shutdown().await {
            eprintln!("Failed to shutdown stream: {}", e);
        }
    }
}

pub async fn serve(port: u16) -> Result<Arc<Server>, Error> {
    let address = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(address).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let server = Arc::new(Server {
        listener,
        closed: AtomicBool::new(false),
    });

    let server_clone = Arc::clone(&server);
    tokio::spawn(Server::listen(server_clone));

    Ok(server)
}
