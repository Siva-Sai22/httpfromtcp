mod headers;
mod request;
mod server;

#[tokio::main]
async fn main() {
    let port = 42069;
    let server = server::serve(port).await.expect("Cannot start server");

    tokio::signal::ctrl_c().await.unwrap();
    println!("Shutting down…");

    server.close();
}
