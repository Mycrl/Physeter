import { KernelCompleteOptions } from "../kernel"
import { hash } from "../../lib/util"
import File from "../../lib/fs"
import Decoder from "./decoder"
import Encoder from "./encoder"
import { join } from "path"

// 内部类型
type TrackIndex = number
type NextIndex = number

/**
 * 索引
 */
export interface Index {
    name: string                               // 名称
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 返回值
 */
export interface Result {
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 内部索引
 */
export interface PrivateIndex {
    key: Buffer                                // 名称摘要
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 索引缓存
 */
export interface IndexCache {
    offset: number                             // 索引偏移位置
    cycle: number                              // 存活时间
    link: number                               // 访问次数
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 索引类
 * @class
 */
export default class {
    private offsets_cache: Map<number, boolean>
    private options: KernelCompleteOptions
    private cache: Map<string, IndexCache>
    private file_size: number
    private file: File

    /**
     * @param options 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.file = new File(join(options.directory, "index"))
        this.offsets_cache = new Map()
        this.cache = new Map()
        this.options = options
        this.file_size = 0
    }
    
    /**
     * 初始化
     * @desc 
     * 初始化文件句柄以及文件描述
     * !!! 外部需要强制调用初始化
     */
    public async initialize() {
        await this.file.initialize()
        this.file_size = (await this.file.stat()).size
    }

    /**
     * 获取索引
     * @param name 名称
     */
    public async get(name: string): Promise<Result | null> {
        const key = hash(name)
        let hit_index = null
        let offset = 0
        
        /**
         * 检查缓存是否存在
         * 如果存在缓存则更新缓存并返回
         */
    if (this.cache.has(name)) {
        let value = this.cache.get(name)!
        value.cycle = Date.now()
        value.link += 1
        return value
    }

        /**
         * 无限循环
         * 直到匹配正确结果或者无法匹配
         */
    for (let i = 0;; i ++) {
        offset = i * 66

        /**
         * 排除掉已经缓存的索引
         * 加快查找速度
         */
        if (this.offsets_cache.has(i * 66)) {
            continue
        }

        /**
         * 从文件流中读取固定长度分片
         * 如果无法读出，所以没有数据
         * 这时候跳出循环
         */
        const buf = Buffer.allocUnsafeSlow(66)
        const size = await this.file.read(buf, i * 66)
        if (size === 0) {
            break
        }

        /**
         * 惰性解码缓冲区
         * 如果没有解码结果
         * 则跳转到下个循环
         */
        const chunk = buf.subarray(0, size)
        const value = Decoder(chunk)
        if (value === null) {
            continue
        }

        /**
         * 对比HASH
         * 这里直接对比Buffer
         * 如果不匹配则跳转下个循环
         */
        if (!key.equals(value.key)) {
            continue
        }

        // 命中索引
        hit_index = value
        break
    }

        /**
         * 如果没有找到索引
         * 则返回Null
         */
        if (!hit_index) {
            return null
        }

        /**
         * 缓存索引
         * 记录存活时间和热度
         */
        this.offsets_cache.set(offset, true)
        this.cache.set(name, {
            start_matedata: hit_index!.start_matedata,
            start_chunk: hit_index!.start_chunk,
            cycle: Date.now(),
            link: 0,
            offset
        })

        return {
            start_matedata: hit_index!.start_matedata,
            start_chunk: hit_index!.start_chunk,
        }
    }

    /**
     * 设置索引 
     * @param index 索引
     * @desc
     * 查找重复索引依赖内存数据实现，
     * 如果索引未存在于内存中会直接写入文件尾部，
     * 此时需要依赖定时碎片整理来合并重复项
     */
    public async set(index: Index): Promise<boolean> {
        const key = hash(index.name)
        
        /**
         * 如果索引已经存在
         * 则返回设置失败
         */
        if (this.cache.has(index.name)) {
            return false
        }

        /**
         * 初始化索引信息
         * 将索引存储到内存缓存
         */
        this.cache.set(index.name, {
            start_matedata: index.start_matedata,
            start_chunk: index.start_chunk,
            offset: this.file_size,
            cycle: Date.now(),
            link: 0
        })

        /**
         * 编码索引数据
         * 追加写入到索引文件中
         */
        const packet = Encoder({ ...index, key })
        await this.file.append(packet)
        this.file_size += 66
        
        return true
    }
}
