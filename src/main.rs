use tokio::net::TcpListener;

mod headers;
mod request;

use crate::request::request_from_reader;

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

                tokio::spawn(async move {
                    match request_from_reader(stream).await {
                        Ok(mut parsed_request) => {
                            println!("Request Line:");
                            println!("- Method: {:?}", parsed_request.request_line.method);
                            println!("- Target: {}", parsed_request.request_line.request_target);
                            println!("- Version: {}", parsed_request.request_line.http_version);

                            println!("Headers:");
                            parsed_request
                                .headers
                                .for_each(|a, b| println!("- {}: {}", a, b));

                            println!("Body:");
                            println!("{}", String::from_utf8(parsed_request.body).unwrap());
                        }
                        Err(e) => eprintln!("Failed to parse request: {}", e),
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
