mod kernel;
mod server;

use kernel::Kernel;
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let kernel = Kernel::new(
        Path::new("./.static"), 
        1024 * 1024 * 1024 * 10
    )?;


    server::run(
        Arc::new(Mutex::new(kernel))
    ).await
}
