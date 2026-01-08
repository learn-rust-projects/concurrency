use tokio::{io::AsyncWriteExt, net::TcpListener};
use tracing::{info, warn};
mod dredis;
use tokio::net::TcpStream;
const BUFF_SIZE: usize = 1024;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("Dredis: listening  on {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Dredis: accepted connection from {}", addr);
        tokio::spawn(async move {
            if let Err(e) = process_redis(stream, addr).await {
                info!("process redis error: {:?}", e);
            }
        });
    }
}
async fn process_redis(mut stream: TcpStream, addr: std::net::SocketAddr) -> anyhow::Result<()> {
    loop {
        // let (mut reader, mut writer) = stream.into_split();
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUFF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => {
                println!("connection closed");
                break;
            }
            Ok(n) => {
                info!("read {} bytes", n);
                let line: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&buf[..n]);
                let _ = stream.write_all(b"+OK\r\n").await;
                info!("read data: {}", line);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 虽然 ready 说可读，但可能被其他任务抢占了数据
                // 或者是虚假唤醒，直接继续循环即可
                continue;
            }
            Err(e) => {
                anyhow::bail!("read error: {:?}", e);
            }
        }
    }
    warn!("process redis loop exit, addr: {}", addr);
    Ok(())
}
