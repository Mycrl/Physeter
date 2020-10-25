# Physeter

![languages](https://img.shields.io/github/languages/top/quasipaa/Physeter)
![open issues](https://img.shields.io/github/issues/quasipaa/Physeter)
![pull requests](https://img.shields.io/github/issues-pr/quasipaa/Physeter)
![license](https://img.shields.io/github/license/quasipaa/Physeter)
![forks](https://img.shields.io/github/forks/quasipaa/Physeter)
![stars](https://img.shields.io/github/stars/quasipaa/Physeter)
![author](https://img.shields.io/badge/author-Mr.Panda-read)

对象存储核心，对于管理大量小文件有非常大的优势，以及优化了WAF（写入放大）等问题，因为磁盘IO的特性，目前只采用了单线程顺序读写的方式，以达到性能最大化的目的.


### 快速开始

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


### 性能
```
单线程顺序写入 `HDD: WDCWD10EZEX 1TB` 180MB/s  
单线程顺序读取 `HDD: WDCWD10EZEX 1TB` 544MB/s (有页缓存)
单线程顺序写入 `SSD: Samsung 860 EVO 250GB` 508MB/s  
单线程顺序读取 `SSD: Samsung 860 EVO 250GB` 1325MB/s (有页缓存)
```


### 概述

数据以固定大小`(4KB)`分片写入轨道文件，使用轨道文件的目的是为了兼容部分文件系统的单文件最大容量，轨道文件头部保存了当前轨道已经释放的块链表，保存尾部的目的是为了链表的快速追加，每个分片内部具有链表形式的下个分片位置以及当前分片内容长度，这虽然会导致一些空间浪费，但这是无法避免的.

```
    
        |-  track header -|                /------------------------------/
        +-----------------+  +-----------------------------+       +----------------------+
        | U64 | U64 | U64 |  | 4KB | 4KB | 4KB | 4KB | 4KB >       | U16 | U64 | * (data) >
        +-----------------+  +-----------------------------+       +----------------------+
            |     |     |-> data size                                  |     |-> next chunk offset
            |     |-> free chunk list last offset                      |-> chunk data size (if full is 0)
            |-> free chunk list first offset
```

轨道内部并不实现文件分配表，文件分配表由外部KV存储维护.

### License
[GPL](./LICENSE)
Copyright (c) 2020 Mr.Panda.