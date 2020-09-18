import { EventEmitter } from "events"
import { freemem } from "os"
import Index from "./index"

/**
 * 核心配置
 */
export interface KernelOptions {
    directory: string             // 存储目录
    track_size?: number           // 轨道文件大小
    chunk_size?: number           // 分片大小
    max_memory?: number           // 最大内存占用
}

/**
 * 核心完整配置
 */
export interface KernelCompleteOptions extends KernelOptions {
    track_size: number            // 轨道文件大小
    chunk_size: number            // 分片大小
    max_memory: number            // 最大内存占用
}

/**
 * 默认配置处理
 * @param options 外部配置
 */
function defaultOptions({ directory, ...options }: KernelOptions): KernelCompleteOptions {
    const max_memory = options.max_memory || Math.floor(freemem() / 2)     // 默认为系统空闲内存一半
    const track_size = options.track_size || 1024 * 1024 * 1024 * 50      // 默认为50G
    const chunk_size = options.chunk_size || 1024 * 4                     // 默认为4KB
    return { directory, max_memory, track_size, chunk_size }
}

/**
 * 核心类
 * @class
 */
export default class extends EventEmitter {
    private options: KernelCompleteOptions
    private index: Index

    /**
     * @param options 配置
     * @constructor
     */
    constructor(options: KernelOptions) {
        super()
        this.options = defaultOptions(options)
        this.index = new Index(this.options)
    }

    /**
     * 读取文件
     * @desc 打开文件读取流
     * @param name 文件名
     */
    public read(name: string) {
        
    }

    /**
     * 写入文件
     * @desc 打开文件写入流
     * @param name 文件名
     */
    public write(name: string) {
        
    }

    /**
     * 删除文件
     * @param name 文件名
     */
    public delete(name: string) {
        
    }
}