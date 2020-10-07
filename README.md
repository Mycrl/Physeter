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


### License
[GPL](./LICENSE)
Copyright (c) 2020 Mr.Panda.