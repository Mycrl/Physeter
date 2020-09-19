import { KernelCompleteOptions } from "./kernel"
import { Readable, Writable } from "stream"
import { exists, readdir } from "../lib/fs"
import { Not } from "../lib/util"
import { Track } from "./track"
import { join } from "path"

// 轨道列表
type Tracks = { [key: number]: Track }

/**
 * 存储类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private tracks: Tracks

    /**
     * @param options 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.options = options
        this.tracks = {}
    }
    
    /**
     * 创建轨道
     * @param track_id 轨道号
     */
    private async create(track_id: number): Promise<Not> {
        this.tracks[track_id] = new Track(track_id, this.options)
        await this.tracks[track_id].initialize()
    }

    /**
     * 初始化
     *  @desc 
     * !!! 外部需要强制调用初始化
     */
    public async initialize(): Promise<Not> {
        const { directory } = this.options

        /**
         * 如果索引文件不存在
         * 则返回空索引
         * 并创建初始轨道
         */
        if (!await exists(join(directory, "index"))) {
            this.tracks[0] = new Track(0, this.options)
            await this.tracks[0].initialize()
        }

        /**
         * 读取目录
         * 查找所有轨道
         */
        const tracks = (await readdir(directory))
        .filter(x => x.endsWith(".track"))
        .map(x => x.replace(".track", ""))
        .map(Number)
        .sort()

        /**
         * 初始化所有轨道
         * 并附加到轨道列表
         */
        for (const track_id of tracks) {
            await this.create(track_id)
        }
    }
    
    /**
     * 读取数据
     * @param tarck 轨道索引
     * @param index 链表头索引
     * @desc 返回可读流
     */
    public read(track: number, index: bigint): Reader {
        return new Reader(this.tracks, track, index)
    }
    
    /**
     * 写入数据
     * @desc 返回可写流
     */
    public write(): Writer {
        return new Writer(this.tracks, this.options, async track_id => {
            await this.create(track_id)
        })
    }
    
    /**
     * 删除数据
     * @param tarck 轨道索引
     * @param index 链表头索引
     */
    public async remove(track: number, index: bigint): Promise<Not> {
for (let track_id = track, offset = index;;) {
        const current_track = this.tracks[track_id]
        const result = await current_track.remove(Number(offset))
        if (result === undefined) break
        track_id = result.next_track!
        offset = result.next!
}
    }
}

/**
 * 可读流
 * @class
 */
class Reader extends Readable {
    private tracks: Tracks
    private track: number
    private index: bigint
    
    /**
     * @param tracks 轨道列表
     * @param track 初始轨道
     * @param index 链表头索引
     * @constructor
     */
    constructor(
        tracks: Tracks, 
        track: number, 
        index: bigint
    ) {
        super()
        this.track = track
        this.index = index
        this.tracks = tracks
    }
    
    /**
     * 读取流
     * @desc 
     * 不关心外部读取长度
     * 总是返回固定长度
     */
    async _read(): Promise<Not> {
        const { track, index } = this
        const value = await this.tracks[track].read(Number(index))
        if (value.next !== undefined) this.index = value.next
        if (value.next_track !== undefined) this.track = value.next_track
        this.push(value.next === undefined ? null : value.data)
    }
}

/**
 * 上个分片
 */
export interface Previous {
    track: number    // 轨道
    index: number    // 索引
    id: number       // ID
    data: Buffer     // 数据
    next?: bigint    // 下个索引
    next_track?: number   // 下个轨道
}

/**
 * 可写流
 * @class
 */
class Writer extends Writable {
    private callback: (x: number) => Promise<Not>
    private options: KernelCompleteOptions
    private write_tracks: Set<number>
    private previous?: Previous 
    private diff_size: number
    private buffer: Buffer
    private index?: bigint
    private first: boolean
    private tracks: Tracks
    private track: number
    private id: number
    
    /**
     * @param options 核心配置
     * @param tracks 轨道列表
     * @param callback 创建轨道回调
     * @constructor
     */
    constructor(
        tracks: Tracks,
        options: KernelCompleteOptions,
        callback: (x: number) => Promise<Not>
    ) {
        super()
        this.diff_size = options.chunk_size - 17
        this.buffer = Buffer.alloc(0)
        this.write_tracks = new Set()
        this.callback = callback
        this.options = options
        this.tracks = tracks
        this.first = false
        this.track = 0
        this.id = 0
    }
    
    /**
     * 分配轨道位置
     * @desc 预先检查轨道写入余量
     */
    private async alloc(): Promise<Not> {
        const { chunk_size, track_size } = this.options
for (;;) {
        
        /**
         * 检查当前轨道文件是否存在
         * 如果不存在则回调给上层创建轨道
         */
        if (this.tracks[this.track] === undefined) {
            await this.callback(this.track)
            break
        }
        
        /**
         * 检查数据写入之后是否溢出轨道
         * 如果溢出则前进到下一个轨道
         * 否则跳出循环
         */
        const { size } = this.tracks[this.track]
        if (size + chunk_size > track_size) {
            this.track += 1
            continue
        } else {
            break
        }
}
    }
    
    /**
     * 写入数据
     * @param chunk 数据
     * @param callback 写入回调
     */
    async _write(
        chunk: Buffer, 
        _: string, 
        callback: (x: null) => Not
    ): Promise<Not> {
        
        /**
         * 附加缓冲区
         * 检查缓冲区长度
         * 如果不满足最低写入要求则跳出
         */
        this.buffer = Buffer.concat([ this.buffer, chunk ])
        if (this.buffer.length < this.diff_size) {
            return callback(null)
        }
        
        /**
         * 无限循环
         * 直到完成当前缓冲区
         */
for (let offset = 0;;) {
        const start = offset * this.diff_size
        const end = start + this.diff_size
    
        /**
         * 检查是否写入到尾部
         * 如果写入到尾部则重新分配缓冲区
         * 并跳出循环结束当前写入
         */
        if (end >= this.buffer.length) {
            this.buffer = this.buffer.slice(end)
            return callback(null)
        }
    
        /**
         * 分配轨道
         * 分配轨道写入索引
         */
        await this.alloc()
        this.write_tracks.add(this.track)
        const current_track = this.tracks[this.track]
        const index = await current_track.alloc()
        
        /**
         * 如果上个分片不为空
         * 则先写入上个分片
         */
        if (this.previous !== undefined) {
            const track = this.tracks[this.previous.track]
            this.previous.next = BigInt(index)
            this.previous.next_track = this.track
            await track.write(this.previous, this.previous.index)
        }
        
        // 准备下个写入的分片 
        this.previous = {
            data: this.buffer.subarray(start, end),
            track: this.track,
            id: this.id,
            index
        }
    
        // 序号叠加
        this.id += 1
}
    }
    
    /**
     * 关闭流
     * @param callback 回调
     */
    async _final(callback: () => Not): Promise<Not> {
        
        /**
         * 尾部处理
         * 如果有未完成的分片
         * 则先写入未完成分片
         */
        if (this.previous !== undefined) {
            const track = this.tracks[this.previous.track]
            await track.write(this.previous, this.previous.index)   
        }
        
        /**
         * 遍历受影响的分片
         * 将每个分片的写入完成
         */
        for (const track_id of this.write_tracks) {
            await this.tracks[track_id].write_end()
        }
        
        callback()
    }
}
