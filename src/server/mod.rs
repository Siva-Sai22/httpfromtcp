use std::pin::Pin;
use std::sync::atomic::Ordering::SeqCst;
use std::{
    io::Error,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::io::{AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::request::{Request, request_from_reader};
use crate::response::{self, StatusCode};

pub type Handler = fn(
    Box<dyn AsyncWrite + Send + Unpin>,
    Request,
) -> Pin<Box<dyn Future<Output = Option<HandlerError>> + Send>>;

pub struct Server {
    handler: Handler,
    listener: TcpListener,
    closed: AtomicBool,
}

pub struct HandlerError {
    pub status_code: response::StatusCode,
    pub message: String,
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
        let mut buf: Vec<u8> = Vec::new();
        let status: StatusCode;

        let request = match request_from_reader(&mut stream).await {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                let _ = response::write_status_line(&mut stream, StatusCode::BadRequest).await;
                return;
            }
        };

        let (writer, mut reader) = tokio::io::duplex(4096);
        let writer_boxed: Box<dyn AsyncWrite + Send + Unpin> = Box::new(writer);

        // Deadlock
        let handler_future = (self.handler)(writer_boxed, request);
        let read_future = reader.read_to_end(&mut buf);

        let (handler_result, _) = tokio::join!(handler_future, read_future);

        match handler_result {
            Some(err) => {
                status = err.status_code;
                buf.extend_from_slice(err.message.as_bytes());
            }
            None => {
                status = response::StatusCode::Ok;
            }
        }

        if let Err(e) = response::write_status_line(&mut stream, status).await {
            eprintln!("Failed to write status line to stream: {}", e);
            return;
        }

        let headers = response::get_default_headers(buf.len() as u16);
        if let Err(e) = response::write_headers(&mut stream, headers).await {
            eprintln!("Failed to write headers to stream: {}", e);
            return;
        }

        if let Err(e) = stream.write_all(buf.as_slice()).await {
            eprintln!("Failed to write body to stream: {}", e);
            return;
        }

        if let Err(e) = stream.shutdown().await {
            eprintln!("Failed to shutdown stream: {}", e);
            return;
        }
    }
}

pub async fn serve(port: u16, handler: Handler) -> Result<Arc<Server>, Error> {
    let address = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(address).await {
        Ok(res) => res,
        Err(e) => return Err(e),
    };

    let server = Arc::new(Server {
        handler,
        listener,
        closed: AtomicBool::new(false),
    });

    let server_clone = Arc::clone(&server);
    tokio::spawn(Server::listen(server_clone));

    Ok(server)
}
