use tokio::io::AsyncWriteExt;

use crate::request::Request;
use crate::server::Writer;

mod headers;
mod request;
mod response;
mod server;

#[tokio::main]
async fn main() {
    let port = 42069;

    let server = server::serve(port, |mut stream: Writer, request: Request| async move {
        if request.request_line.request_target == "/yourproblem" {
            return Some(server::HandlerError {
                status_code: response::StatusCode::InternalServerError,
                message: "Your problem is too complex.".to_string(),
            });
        } else if request.request_line.request_target == "/myproblem" {
            return Some(server::HandlerError {
                status_code: response::StatusCode::InternalServerError,
                message: "Woopsie, my bad!\n".to_string(),
            });
        }

        stream.write_all(b"All Good! frfr\n").await.unwrap();
        None
    })
    .await
    .expect("Cannot start server");

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down…");

    server.close();
}
