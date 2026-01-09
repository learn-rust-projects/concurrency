use simple_redis::backend::Backend;
use tokio::net::TcpListener;
use tracing::{info, warn};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: listening  on {}", addr);
    let backend = Backend::new();
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Dredis: accepted connection from {}", addr);
        let backend = backend.clone();
        tokio::spawn(async move {
            if let Err(e) = simple_redis::network::stream_handler(stream, backend).await {
                warn!("process redis error: {:?}", e);
            }
        });
    }
}
