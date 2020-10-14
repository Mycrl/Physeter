# Physeter

![languages](https://img.shields.io/github/languages/top/quasipaa/Physeter)
![open issues](https://img.shields.io/github/issues/quasipaa/Physeter)
![pull requests](https://img.shields.io/github/issues-pr/quasipaa/Physeter)
![license](https://img.shields.io/github/license/quasipaa/Physeter)
![forks](https://img.shields.io/github/forks/quasipaa/Physeter)
![stars](https://img.shields.io/github/stars/quasipaa/Physeter)
![author](https://img.shields.io/badge/author-Mr.Panda-read)

这是基于Rust编程语言的对象存储服务器，项目创建的目的是为了解决作者本人管理大量媒体文件的困扰，所以便开始自行创建一个简单的对象存储服务器，用于支持在低功耗设备运行（比如树莓派），并同时提供不错的性能，因为操作系统提供的文件系统对比管理海量文件性能有限，以及难以有效合并分散于多个磁盘的文件，所以这个项目使用集中管理文件内容的方式组织多个文件，以及支持多磁盘和多位置合并，为了提高读写速度，本项目使用多线程同时写入多个文件，将读写负载分散到多个目标文件来获得并行读写速度，而且对于SSD固态存储的写入放大（WAF）等等问题也做了相应优化.  

> 本项目作为流媒体服务器[Spinosa](https://github.com/quasipaa/Spinosa)的附属项目，同样也是为了解决流媒体服务器的静态存储问题，这是一个必要的附属服务.


### 版本
目前还在实现中，无测试版本.


### 概述

- #### Track
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

轨道内部并不实现文件分配表，文件分配表由外部KV存储维护，轨道文件可以存储在不同位置以至于可以存储到不同磁盘，但是不影响索引合并，这是为了现实情况需要将文件存储在不同位置而存在的特性，当文件存储在不同磁盘时，会为每个磁盘指定一个单独的线程执行读写操作，这样可以最大化利用多磁盘IO叠加.

- #### Node
单个存储分区实例由一个索引管理多个轨道文件，索引由`RocksDB`实现，只做单层的文件索引，对于文件夹或者存储桶的行为考虑，当个节点内部会维护多个分区实例，每个分区对应不同的存储桶，节点处理网络数据的流入流出，并平衡多个分区实例的索引分布，对于多个网络数据流的情况，节点会使用轮询机制保证每个数据流的平衡和合理性，不会让单个流始终抢占当前任务队列，虽然这种分散式的写入读取行为会导致磁盘IO性能有一定下降，但是对于用户体验性的提升是可预见的.

```
                +-------------------------------+        +-------------------------------+
                | Track | Track | Track | Track |        | Track | Track | Track | Track |
                +-------------------------------+        +-------------------------------+
                |- RocksDB (Index / Alloc Map) -|        |- RocksDB (Index / Alloc Map) -|
                |-                                 Node                                 -|
```

多个节点组成分布式集群，多个节点之间使用`Raft`一致性共识协议来管理分区平衡表，通过平衡表快速找到当前文件所在节点，每个节点的索引数据并不共享，自身维护可用性，当前对于复制节点的支持还没有提上日程，因为目前这不是一个高优先级的功能.


### 计划
* [x] 存储分区核心   
* [ ] 多磁盘线程分离  
* [ ] 多索引分区平衡机制  
* [ ] 存储桶  
* [ ] 外部网络接口  
* [ ] Raft支持

### License
[GPL](./LICENSE)
Copyright (c) 2020 Mr.Panda.