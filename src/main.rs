mod kernel;

use anyhow::Result;
use kernel::Kernel;
use std::time::Instant;
use std::path::Path;

fn main() -> Result<()> {
    let mut kernel = Kernel::new(
        Path::new("./.static"), 
        1024 * 1024 * 1024 * 1
    )?;

    let writer = std::fs::File::create("./testing/output.mp4")?;
    let reader = std::fs::File::open("./testing/末代皇帝.mp4")?;

    let start = Instant::now();
    kernel.write(b"test", reader)?;
    println!("write cost: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.read(b"test", writer)?;
    println!("read cost: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.delete(b"test")?;
    println!("delete cost: {:?} ms", start.elapsed().as_millis());

    Ok(())
}
