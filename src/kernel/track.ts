import ChunkClass, { Chunk, Result, LazyResult } from "./chunk"
import { KernelCompleteOptions } from "./kernel"
import { Not } from "../lib/util"
import File from "../lib/fs"
import { join } from "path"

/**
 * 轨道类
 * @class
 */
export class Track {
    private options: KernelCompleteOptions
    private free_start: number
    private free_end: number
    private chunk: ChunkClass
    private file: File
    private id: number
    public size: number

    /**
     * @param id 轨道ID
     * @param options 核心配置
     * @constructor
     */
    constructor(id: number, options: KernelCompleteOptions) {
        this.file = new File(join(options.directory, `${id}.track`))
        this.chunk = new ChunkClass(options)
        this.options = options
        this.free_start = 0
        this.free_end = 0
        this.size = 0
        this.id = id
    }
    
    /**
     * 创建默认文件头
     * @desc 创建默认的失效头尾索引
     */
    private async default_header(): Promise<Not> {
        const free_buf = Buffer.allocUnsafeSlow(16)
        free_buf.writeBigInt64BE(0n, 0)
        free_buf.writeBigInt64BE(0n, 8)
        await this.file.write(free_buf, 0)
        this.size = 16
    }
    
    /**
     * 读取失效块索引
     * @desc
     * 如果不存在就创建0索引
     * 如果存在就读取索引
     */
    private async read_free(): Promise<Not> {
        if (this.size === 0) return await this.default_header()
        const free_buf = Buffer.allocUnsafeSlow(16)
        await this.file.read(free_buf, 0)
        const start = free_buf.readBigInt64BE(0)
        const end = free_buf.readBigInt64BE(8)
        this.free_start = Number(start)
        this.free_end = Number(end)
    }
    
    /**
     * 初始化
     * @desc 
     * !!! 外部需要强制调用初始化
     */
    public async initialize(): Promise<Not> {
        await this.file.initialize()
        this.size = (await this.file.stat()).size
        await this.read_free()
    }
    
    /**
     * 删除数据
     * @param index 头部索引
     */
    public async remove(index: number): Promise<Not | LazyResult> {
        const { chunk_size } = this.options
        const free_byte = Buffer.from([0])
for (let offset = index, i = 0;; i ++) {

        /**
         * 遍历完文件
         * 跳出循环
         */
        if (offset >= this.options.track_size) {
            break
        }

        /**
         * 读取分片
         * 如果没有数据则跳出
         */
        let chunk = Buffer.allocUnsafeSlow(chunk_size)
        const size = await this.file.read(chunk, offset)
        if (size === 0) {
            break
        }
    
        /**
         * 轨道数据长度减去单分片长度
         * 更改状态位为失效并解码当前分片
         */
        this.size -= chunk_size
        await this.file.write(free_byte, offset + 4)
        const value = this.chunk.lazy_decoder(chunk)
        
        /**
         * 如果失效索引头未初始化
         * 则先初始化索引头
         */
        if (this.free_start === 0) {
            const next_buf = Buffer.allocUnsafeSlow(8)
            next_buf.writeBigInt64BE(BigInt(offset))
            await this.file.write(next_buf, 0)
            this.free_start = offset
        }
    
        /**
         * 如果尾部索引已存在
         * 则将当前尾部和现在的分片索引连接
         */
        if (this.free_end > 0 && i === 0) {
            const next_buf = Buffer.allocUnsafeSlow(8)
            next_buf.writeBigInt64BE(BigInt(offset))
            await this.file.write(next_buf, this.free_end + 7)
        }
    
        /**
         * 如果下个索引为空
         * 则表示分片列表已到尾部
         * 更新失效索引尾部
         * 跳出循环
         */
        if (value.next === undefined) {
            const end_buf = Buffer.allocUnsafeSlow(8)
            end_buf.writeBigInt64BE(BigInt(offset))
            await this.file.write(end_buf, 0)
            this.free_end = offset
            break
        }
        
        // 更新索引
        offset = Number(value.next)

        /**
         * 下个索引不在这个轨道文件
         * 转移到其他轨道继续流程
         */
        if (value.next_track !== this.id) {
            return value
        }
}
    }
    
    /**
     * 分配分片位置
     * @desc 
     * 只计算偏移
     * 并不会实际写入数据
     */
    public async alloc(): Promise<number> {
        const { chunk_size } = this.options
        
        /**
         * 没有失效块
         * 直接写入尾部
         */
        if (this.free_start == 0) {
            return this.size
        }

        /**
         * 读取失效分片
         * 并解码失效分片
         */
        let free_buf = Buffer.allocUnsafeSlow(chunk_size)
        await this.file.read(free_buf, this.free_start)
        const value = this.chunk.lazy_decoder(free_buf)
        
        
        /**
         * 如果还有失效分片
         * 则更新链表头部为下个分片位置
         * 如果失效分片已经全部解决
         * 则归零链表头部
         */
        const free_start = this.free_start 
        this.free_start = Number(value.next) || 0
    
        /**
         * 归零链表头部时
         * 也同时归零链表尾部
         * 因为已无失效分片
         */
        if (this.free_start === 0) {
            this.free_end = 0
        }
        
        return free_start
    }
    
    /**
     * 写入分片
     * @param chunk 分片
     * @param index 分片索引
     */
    public async write(chunk: Chunk, index: number): Promise<Not> {
        const packet = this.chunk.encoder(chunk)
        await this.file.write(packet, index)
    }
    
    /**
     * 写入结束
     * @desc 
     * 将状态转储到磁盘
     * 写入完成之后必须调用
     */
    public async write_end(): Promise<Not> {
        let packet = Buffer.allocUnsafeSlow(16)
        packet.writeBigInt64BE(BigInt(this.free_start), 0)
        packet.writeBigInt64BE(BigInt(this.free_end), 8)
        await this.file.write(packet, 0)
    }
    
    /**
     * 读取分片数据
     * @param index 分片索引
     */
    public async read(index: number): Promise<Result> {
        let packet = Buffer.allocUnsafeSlow(this.options.chunk_size)
        await this.file.read(packet, index)
        return this.chunk.decoder(packet)
    }
}
