mod kernel;

use kernel::{Kernel, KernelOptions};
use std::time::Instant;
use std::path::Path;
use anyhow::Result;

fn main() -> Result<()> {
    let mut kernel = Kernel::new(KernelOptions {
        directory: Path::new("./data"),
        track_size: 1024 * 1024 * 1024 * 5,
        max_memory: 1024 * 1024 * 1024,
        chunk_size: 1024 * 4,
    })?;

    kernel.open()?;

    let writer = std::fs::File::create("./output.mp4")?;
    let reader = std::fs::File::open("./末代皇帝.mp4")?;

    let start = Instant::now();
    kernel.write("test", reader)?;
    println!("write cost: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.read("test", writer)?;
    println!("read cost: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.delete("test")?;
    println!("delete cost: {:?} ms", start.elapsed().as_millis());

    kernel.shutdown()?;
    Ok(())
}
