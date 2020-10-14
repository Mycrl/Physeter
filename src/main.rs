mod kernel;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // let (sender, reader) = mpsc::channel(1);
    server::run().await
}
