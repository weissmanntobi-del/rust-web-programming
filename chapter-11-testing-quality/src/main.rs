use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, chapter_11_testing_quality::app()).await?;
    Ok(())
}
