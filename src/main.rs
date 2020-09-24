mod kernel;

use kernel::{Kernel, KernelOptions, fs};
use std::time::Instant;
use std::path::Path;
use anyhow::Result;

fn main() -> Result<()> {
    let mut kernel = Kernel::new(KernelOptions {
        directory: Path::new("./data"),
        track_size: 1024 * 1024 * 1024 * 50,
        max_memory: 1024 * 1024 * 1024,
        chunk_size: 1024 * 4,
    })?;

    kernel.open()?;

    let mut file = fs::Fs::new(Path::new("./活着.mp4"))?;
    let mut offset = 0;

    let start = Instant::now();
    if let Some(mut writer) = kernel.write("test")? {
        loop {
            let mut buffer = [0u8; 2048];
            let size = file.read(&mut buffer, offset)?;
    
            offset += size as u64;
            writer.write(if size == 0 {
                None
            } else {
                Some(&buffer[0..size])
            })?;

            if size == 0 {
                break;
            }
        }
    }

    println!("time cost: {:?} ms", start.elapsed().as_millis());
    kernel.shutdown()?;

    Ok(())
}
