import { KernelCompleteOptions } from "./kernel"

/**
 * 分片
 */
export interface Chunk {
    id: number                  // 分片ID
    next?: bigint               // 下个分片索引
    next_track?: number         // 下个分片轨道
    data: Buffer                // 数据
}

/**
 * 分片返回实例
 * @desc 分片列表以链表形式存储
 */
export interface Result {
    id: number                  // 分片ID
    exist: boolean              // 分片是否有效
    next?: bigint               // 下个分片索引
    next_track?: number         // 下个分片轨道
    data: Buffer                // 数据
}

/**
 * 惰性返回值
 * @desc 惰性解码返回
 */
export interface LazyResult {
    next?: bigint               // 下个分片索引
    next_track?: number         // 下个分片轨道
}

/**
 * 分片类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private diff_size: number
    
    /**
     * @param options 配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.diff_size = options.chunk_size - 17
        this.options = options
    }
    
    /**
     * 编码分片
     * @param chunk 分片
     */
    public encoder(chunk: Chunk): Buffer {
        let packet = Buffer.alloc(this.options.chunk_size)
        packet.writeInt32BE(chunk.id, 0)
        packet.writeInt8(1, 4)
        packet.writeInt16BE(this.diff_size ? 0 : chunk.data.length, 5)
        packet.writeBigInt64BE(chunk.next || 0n, 7)
        packet.writeInt16BE(chunk.next_track || 0, 15)
        chunk.data.copy(packet, 17)
        return packet
    }
    
    /**
     * 解码分片
     * @param chunk Buffer
     */
    public decoder(chunk: Buffer): Result {
        const id = chunk.readInt32BE(0)
        const _exist = chunk.readInt8(4)
        const size = chunk.readInt16BE(5)
        const _next = chunk.readBigInt64BE(7)
        const _next_track = chunk.readInt16BE(15)
        const data = chunk.subarray(17, size === 0 ? this.diff_size : size)
        const exist = _exist === 1
        const next = _next === 0n ? undefined : _next
        const next_track = _next_track === 0 ? undefined : _next_track
        return { id, exist, next, next_track, data }
    }
    
    /**
     * 惰性解码分片
     * @param chunk Buffer
     * @desc 用于快速遍历索引
     */
    public lazy_decoder(chunk: Buffer): LazyResult {
        const _next = chunk.readBigInt64BE(7)
        const _next_track = chunk.readInt16BE(15)
        const next = _next === 0n ? undefined : _next
        const next_track = _next_track === 0 ? undefined : _next_track
        return { next, next_track }
    }
}
