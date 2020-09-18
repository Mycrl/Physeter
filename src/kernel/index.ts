import { KernelCompleteOptions } from "./kernel"
import { hash, Not } from "../lib/util"
import Queue from "../lib/queue"
import File from "../lib/fs"
import { join } from "path"

// 内部类型
type NextIndex = bigint
type TrackIndex = number
type ParseHandle = (value: PrivateIndex, offset: number) => boolean

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
interface PrivateIndex {
    key: Buffer                                // 名称摘要
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 索引缓存
 */
interface IndexCache {
    offset: number                             // 索引偏移位置
    cycle: number                              // 存活时间
    link: number                               // 访问次数
    start_chunk: [TrackIndex, NextIndex]       // 分片索引列表
    start_matedata: [TrackIndex, NextIndex]    // 文件信息索引列表
}

/**
 * 解码器
 * @param chunk 数据
 */
function Decoder(chunk: Buffer): PrivateIndex | null {
    if (chunk.length !== 54) return null
    if (chunk.readUInt16BE(0) !== 0x9900) return null
    const key = chunk.subarray(2, 34)
    const matedata_track = chunk.readInt16BE(34)
    const matedata_index = chunk.readBigInt64BE(36)
    const chunk_track = chunk.readInt16BE(44)
    const chunk_index = chunk.readBigInt64BE(46)
    const start_chunk = <PrivateIndex["start_matedata"]>[matedata_track, matedata_index]
    const start_matedata = <PrivateIndex["start_chunk"]>[chunk_track, chunk_index]
    return { key, start_chunk, start_matedata }
}

/**
 * 编码器
 * @param index 索引数据
 */
function Encoder(index: PrivateIndex): Buffer {
    let buffer = Buffer.allocUnsafeSlow(54)
    buffer.writeUInt16BE(0x9900, 0)
    index.key.copy(buffer, 2)
    buffer.writeInt16BE(index.start_matedata[0], 34)
    buffer.writeBigInt64BE(index.start_matedata[1], 36)
    buffer.writeInt16BE(index.start_chunk[0], 44)
    buffer.writeBigInt64BE(index.start_chunk[1], 46)
    return buffer
}

/**
 * 索引类
 * @class
 */
export default class extends Queue<Index, boolean> {
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
        const { directory } = options
        super(1)        
        this.file_size = 0
        this.options = options
        this.cache = new Map()
        this.offsets_cache = new Map()
        this.file = new File(join(directory, "index"))
    }
    
    /**
     * 解析索引文件
     * @param handle 处理函数
     */
    private async parse(handle: ParseHandle): Promise<Not> {
for (let i = 0;; i ++) {
        const offset = i * 54

        /**
         * 排除掉已经缓存的索引
         * 加快查找速度
         */
        if (this.offsets_cache.has(offset)) {
            continue
        }

        /**
         * 从文件流中读取固定长度分片
         * 如果无法读出，所以没有数据
         * 这时候跳出循环
         */
        const buf = Buffer.allocUnsafeSlow(54)
        const size = await this.file.read(buf, offset)
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
         * 传递给处理函数
         * 如果返回true则停止循环
         */
        if (handle(value, offset)) {
            break
        }
}
    }
    
    /**
     * 加载索引
     * @desc 将所有索引加载到内存
     */
    private async load_all(): Promise<Not> {
await this.parse(({ key, start_matedata, start_chunk }, offset) => {
        const value = { start_matedata, start_chunk, cycle: Date.now(), link: 0, offset }
        this.cache.set(key.toString("hex"), value)
        this.offsets_cache.set(offset, true)
        return false
})
    }
    
    /**
     * 设置索引 
     * @param index 索引
     * @desc
     * 查找重复索引依赖内存数据实现，
     * 如果索引未存在于内存中会直接写入文件尾部，
     * 此时需要依赖定时碎片整理来合并重复项
     */
    private async set_index(index: Index): Promise<boolean> {
        const key = hash(index.name)
        const key_hex = key.toString("hex")
        
        /**
         * 如果索引已经存在
         * 则返回设置失败
         */
        if (this.cache.has(key_hex)) {
            return false
        }

        /**
         * 初始化索引信息
         * 将索引存储到内存缓存
         */
        this.offsets_cache.set(this.file_size, true)
        this.cache.set(key_hex, {
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
        this.file_size += 54
        
        return true
    }
    
    /**
     * 初始化
     * @desc 
     * 初始化文件句柄以及文件描述
     * !!! 外部需要强制调用初始化
     */
    public async initialize(): Promise<Not> {
        await this.file.initialize() 
        this.file_size = (await this.file.stat()).size
        this.bind(this.set_index.bind(this))
        await this.load_all()
    }

    /**
     * 获取索引
     * @param name 名称
     */
    public async get(name: string): Promise<Result | null> {
        const key = hash(name)
        const key_hex = key.toString("hex")
        
        /**
         * 检查缓存是否存在
         * 如果存在缓存则更新缓存并返回
         */
    if (this.cache.has(key_hex)) {
        let value = this.cache.get(key_hex)!
        value.cycle = Date.now()
        value.link += 1
        return value
    }

        /**
         * 无限循环
         * 直到匹配正确结果或者无法匹配
         */
        let offset = 0
        let hit: PrivateIndex | null = null
await this.parse((value, index) => {
        offset = index

        /**
         * 对比HASH
         * 这里直接对比Buffer
         * 如果不匹配则跳转下个循环
         */
        if (!key.equals(value.key)) {
            return false
        }

        // 命中索引
        hit = value
        return true
})

        /**
         * 如果没有找到索引
         * 则返回Null
         */
        if (!hit) {
            return null
        }

        /**
         * 缓存索引
         * 记录存活时间和热度
         */
        this.offsets_cache.set(offset, true)
        this.cache.set(key_hex, {
            start_matedata: hit!.start_matedata,
            start_chunk: hit!.start_chunk,
            cycle: Date.now(),
            link: 0,
            offset
        })

        return {
            start_matedata: hit!.start_matedata,
            start_chunk: hit!.start_chunk,
        }
    }
    
    /**
     * 设置索引
     * @param index 索引
     */
    public async set(index: Index): Promise<boolean> {
        return await this.call(index)
    }
    
    /**
     * 删除索引
     * @param name 名称
     */
    public async remove(name: string): Promise<boolean> {
        return await this.cache.delete(name)
    }
}
