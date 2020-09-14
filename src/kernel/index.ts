import { KernelCompleteOptions } from "./kernel"
import { createReadStream } from "fs"
import { join } from "path"

/**
 * Index
 * +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-
 * | fixed_header | size    | fixed_header | name_size  | name | metadata_list_size | (Track) | chunk_list_size | (Track) |
 * | u16          | u32     | u16          | u8         | *    | u32                | *       | u32             | *       |
 * +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-
 * 
 * Track
 * +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 * | track_list_size | track | chunk_index |
 * | u32             | u32   | u64         |
 * +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 */

/**
 * 轨道索引
 */
export interface IndexTrack {
    id: number              // 轨道
    index: Array<number>    // 索引列表
}

/**
 * 索引
 */
export interface Index {
    name: string                        // 名称
    chunk_list: Array<IndexTrack>       // 分片索引列表
    matedata_list: Array<IndexTrack>    // 文件信息索引列表
}

/**
 * 内部轨道索引
 */
interface PrivateIndexTrack extends IndexTrack {
    count: number   // 总长度
}

/**
 * 内部索引
 */
interface PrivateIndex extends Index {
    count: number                               // 总长度
    chunk_size: number                          // 分片总长度
    matedata_size: number                       // 文件信息总长度
    chunk_list: Array<PrivateIndexTrack>        // 分片索引列表
    matedata_list: Array<PrivateIndexTrack>     // 文件信息索引列表
}

/**
 * 编码器
 * @class
 */
class Encoder {

    /**
     * 轨道长度计算
     * @param {track} 轨道列表
     */
    private static compute_track_size(track: IndexTrack): PrivateIndexTrack {
        const count = track.index.length * 8 + 4 + 4
        return { ...track, count }
    }

    /**
     * 计算轨道列表总长度
     * @param {tracks} 轨道列表
     */
    private static compute_tracks_count(tracks: Array<PrivateIndexTrack>): number {
        return tracks.map(x => x.count).reduce((x, y) => x + y)
    }

    /**
     * 编码器长度计算
     * @param {index} 索引数据
     */
    private static compute_size(index: Index): PrivateIndex {
        const matedata_list = index.matedata_list.map(Encoder.compute_track_size)
        const chunk_list = index.chunk_list.map(Encoder.compute_track_size)
        const chunk_size = Encoder.compute_tracks_count(chunk_list)
        const matedata_size = Encoder.compute_tracks_count(matedata_list)
        const count = 9 + index.name.length + matedata_size + chunk_size
        return { ...index, count, matedata_list, chunk_list, chunk_size, matedata_size }
    }

    /**
     * 编码器
     * @param {index} 索引数据
     */
    public static parse(index: Index): Buffer {
        let attribute = Encoder.compute_size(index)
        let buffer = Buffer.alloc(attribute.count)
        let offset = 0
        
        // 写入固定位
        buffer.writeUInt16BE(0x9900, offset)
        offset += 2
        
        // 写入总长度
        buffer.writeInt32BE(attribute.count, offset)
        offset += 4
        
        // 写入固定位
        buffer.writeUInt16BE(0x9900, offset)
        offset += 2
        
        // 写入名称长度
        buffer.writeInt8(attribute.name.length, offset)
        offset += 1
        
        // 写入名称
        buffer.write(attribute.name, offset)
        offset += attribute.name.length

        // 写入文件信息总长度
        buffer.writeInt32BE(attribute.matedata_size, offset)
        offset += 4
        
        // 写入文件信息列表
for (const track of attribute.matedata_list) {
        buffer.writeInt32BE(track.count, offset)
        offset += 4
        buffer.writeInt32BE(track.id, offset)
        offset += 4
    for (const index of track.index) {
        buffer.writeBigInt64BE(<unknown>index as bigint, offset)
        offset += 8
    }
}

        // 写入分片索引列表总长度
        buffer.writeInt32BE(attribute.chunk_size, offset)
        offset += 4

        // 写入分片索引列表
for (const track of attribute.chunk_list) {
        buffer.writeInt32BE(track.count, offset)
        offset += 4
        buffer.writeInt32BE(track.id, offset)
        offset += 4
    for (const index of track.index) {
        buffer.writeBigInt64BE(<unknown>index as bigint, offset)
        offset += 8
    }
}

        return buffer
    }
}

/**
 * 解码器
 * @class
 */
class Decoder {
    private buffer: Buffer
    private offset: number
    
    /**
     * @constructor
     */
    constructor() {
        this.buffer = Buffer.alloc(0)
        this.offset = 0
    }
    
    /**
     * 释放缓冲区
     */
    public free() {
        this.buffer = Buffer.alloc(0)
        this.offset = 0
    }
    
    /**
     * 解码
     * @param {chunk} 分片
     */
    public parse(chunk: Buffer) {
        this.buffer = Buffer.concat([ this.buffer, chunk ])
        
    }
    
    /**
     * 惰性解码
     * @param {chunk} 分片
     * @desc
     * 惰性解码是为了加快解码速度，
     * 并不解码全部消息，只解码字符串长度
     */
    public lazy_parse(chunk: Buffer) {
        this.buffer = Buffer.concat([ this.buffer, chunk ])
    }
}

/**
 * 索引缓存
 */
interface IndexCache {
    offset: number                     // 索引偏移位置
    cycle: number                      // 存活时间
    chunk_list: Array<IndexTrack>      // 分片索引列表
    matedata_list: Array<IndexTrack>   // 文件信息索引列表
}

/**
 * 索引内容
 */
interface Value {
    chunk_list: Array<IndexTrack>      // 分片索引列表
    matedata_list: Array<IndexTrack>   // 文件信息索引列表
}

/**
 * 索引类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private cache: Map<string, IndexCache>
    private offsets_cache: Map<number, boolean>
    private decoder: Decoder
    
    /**
     * @param {options} 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.cache_offsets = new Set()
        this.decoder = new Decoder()
        this.options = options
        this.cache = new Map()
    }
    
    /**
     * 获取索引
     * @param {name} 名称
     */
    public get(name: string): Value {
        const { directory } = this.options
        const index_filename = join(directory, "index")
        const readable = createReadStream(index_filename)
        
        /**
         * 检查缓存是否存在
         * 如果存在缓存则直接返回缓存内容
         */
        if (this.cache.has(name))
            return this.cache.get(name)!

        this.decoder.free()
        for(;;) {
            const chunk = readable.read(1024)
           
            if (chunk === null) {
                break
            }
        
            this.decoder.lazy_parse(chunk)
        }
    }
}
