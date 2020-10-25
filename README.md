# Physeter

![languages](https://img.shields.io/github/languages/top/quasipaa/Physeter)
![open issues](https://img.shields.io/github/issues/quasipaa/Physeter)
![pull requests](https://img.shields.io/github/issues-pr/quasipaa/Physeter)
![license](https://img.shields.io/github/license/quasipaa/Physeter)
![forks](https://img.shields.io/github/forks/quasipaa/Physeter)
![stars](https://img.shields.io/github/stars/quasipaa/Physeter)
![author](https://img.shields.io/badge/author-Mr.Panda-read)

This is an object storage core library that has great advantages in managing a large number of small files and optimizing WAF (write amplification) and other issues. Due to the characteristics of disk IO, currently only single-threaded sequential read and write methods are used to maximize read and write performance.


### Quick start

```rust
use physeter::Kernel;
use std::fs::File;

fn main() {
    let mut kernel = Kernel::new(
        "./".to_string(), 
        1024 * 1024 * 1024 * 5
    ).unwrap();
    
    let reader = File::open("./input.mp4").unwrap();
    let writer = File::open("./output.mp4").unwrap();
    
    kernel.write(b"test", reader).unwrap();
    kernel.read(b"test", writer).unwrap();
    kernel.delete(b"test").unwrap();
}
```


### Performance
```
Single thread sequential write `HDD: WDCWD10EZEX 1TB` 180MB/s  
Single thread sequential read `HDD: WDCWD10EZEX 1TB` 544MB/s
Single thread sequential write `SSD: Samsung 860 EVO 250GB` 508MB/s  
Single thread sequential read `SSD: Samsung 860 EVO 250GB` 1325MB/s
```


### Overview

The Data is written into the trace file in `(4KB)`fixed-size segments. The purpose of using trace files is to be compatible with the maximum capacity of a single file in some file systems. The beginning of the track file stores the beginning and end of the linked list of released blocks in the current track. Each fragment saves the position of the next fragment and the length of the current fragment content in the form of a linked list. Although this will cause some space waste, it is inevitable.

```
    
        |-  track header -|                /------------------------------/
        +-----------------+  +-----------------------------+       +----------------------+
        | U64 | U64 | U64 |  | 4KB | 4KB | 4KB | 4KB | 4KB >       | U16 | U64 | * (data) >
        +-----------------+  +-----------------------------+       +----------------------+
            |     |     |-> data size                                  |     |-> next chunk offset
            |     |-> free chunk list last offset                      |-> chunk data size (if full is 0)
            |-> free chunk list first offset
```

There is no file allocation table in the track, this table is maintained by external KV storage.


### License
[GPL](./LICENSE)
Copyright (c) 2020 Mr.Panda.