mod chunk;
mod disk;
mod fs;
mod index;
mod track;

use std::path::Path;

pub struct KernelOptions<'a> {
    directory: &'a Path,
    track_size: u64,
    chunk_size: u64,
    max_memory: u64,
}

impl<'a> Default for KernelOptions<'a> {
    fn default() -> Self {
        Self {
            directory: Path::new("./"),
            track_size: 1024 * 1024 * 1024 * 50,
            max_memory: 1024 * 1024 * 1024,
            chunk_size: 1024 * 4,
        }
    }
}
