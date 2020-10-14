mod kernel;

use kernel::Kernel;
use std::time::Instant;
use std::path::Path;

pub struct Reader {
    size: usize
}

impl Reader {
    pub fn new() -> Self {
        Self {
            size: 0
        }
    }
}

impl std::io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        Ok(if self.size >= 10737418240 {
            0
        } else {
            let size = buf.len();
            self.size += size;
            size
        })
    }
}

pub struct Writer {}

impl std::io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let size = buf.len();
        let _ = buf;
        Ok(size)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut kernel = Kernel::new(
        Path::new("./.static"), 
        1024 * 1024 * 1024 * 5
    )?;

    let writer =  Writer {};
    let reader = Reader::new();

    let start = Instant::now();
    kernel.write(b"test", reader)?;
    println!("write time: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.read(b"test", writer)?;
    println!("read time: {:?} ms", start.elapsed().as_millis());

    let start = Instant::now();
    kernel.delete(b"test")?;
    println!("delete time: {:?} ms", start.elapsed().as_millis());

    Ok(())
}