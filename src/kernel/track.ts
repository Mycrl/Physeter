import { KernelCompleteOptions } from "./kernel"
import { Not } from "../lib/util"
import File from "../lib/fs"
import { join } from "path"
import Chunk from "./chunk"

/**
 * 轨道类
 * @class
 */
export class Track {
    private options: KernelCompleteOptions
    private free_start: number
    private free_end: number
    private chunk: Chunk
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
        this.chunk = new Chunk(options)
        this.options = options
        this.free_start = 0
        this.free_end = 0
        this.size = 0
        this.id = id
    }
    
    /**
     * 初始化
     * @desc 
     * !!! 外部需要强制调用初始化
     */
    public async initialize(): Promise<Not> {
        await this.file.initialize()
        this.size = (await this.file.stat()).size
        await this.read_free_index()
    }
    
    /**
     * 读取失效块索引
     * @desc
     * 如果不存在就创建0索引
     * 如果存在就读取索引
     */
    public async read_free_index(): Promise<Not> {
        const free_buf = Buffer.allocUnsafeSlow(16)
        
        /**
         * 链表头部索引还未初始化
         * 填充默认值初始化链表头部
         */
    if (this.size === 0) {
        free_buf.writeBigInt64BE(0n, 0)
        free_buf.writeBigInt64BE(0n, 8)
        await this.file.write(free_buf, 0)
        this.size = 16
        return undefined
    }
        
        /**
         * 已经初始化
         * 直接读取链表头部索引
         */
        await this.file.read(free_buf, 0)
        const start = free_buf.readBigInt64BE(0)
        const end = free_buf.readBigInt64BE(8)
        this.free_start = Number(start)
        this.free_end = Number(end)
    }
    
    /**
     * 删除数据
     * @param index 头部索引
     */
    public async remove(index: bigint): Promise<Not | number> {
        const { chunk_size } = this.options
        const free_byte = Buffer.from([0])
for (let offset = Number(index), i = 0;; i ++) {

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
         * 更改状态位为失效
         * 解码分片并
         */
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
            return value.next_track
        }
}
    }
    
    /**
     * 写入数据分片
     * @param next_track 轨道ID
     * @param id 分片ID
     * @param data 数据
     * @desc
     * 写入接口只开放给写入流
     * 不考虑全部一次性写入
     */
    public async write(next_track = this.id, id: number, data: Buffer) {
        const { chunk_size } = this.options
        
        /**
         * 检查是否存在失效块
         * 因为失效块索引不可能为0
         * 所以此处检查是否为0
         */
        if (this.free_start == 0) {
            const next = BigInt(this.size + chunk_size)
            const chunk = { next, next_track, data, id }
            const buf = this.chunk.encoder(chunk)
            await this.file.write(buf, this.free_start)
            this.size += chunk_size
            return undefined
        }

        /**
         * 读取失效分片
         * 并解码失效分片
         */
        let free_buf = Buffer.allocUnsafeSlow(chunk_size)
        await this.file.read(free_buf, this.free_start)
        const value = this.chunk.lazy_decoder(free_buf)

        /**
         * 编码分片
         * 写入分片
         */
        await this.file.write(this.chunk.encoder({
            next: value.next || BigInt(this.size),
            next_track, data, id
        }), this.free_start)
        
        /**
         * 如果还有失效分片
         * 则更新链表头部为下个分片位置
         * 如果失效分片已经全部解决
         * 则归零链表头部
         */
        this.free_start = Number(value.next) || 0
    
        /**
         * 归零链表头部时
         * 也同时归零链表尾部
         * 因为已无失效分片
         */
        if (this.free_start === 0) {
            this.free_end = 0
        }
    }
    
    /**
     * 写入结束
     * @desc 将状态转储到磁盘
     */
    public async write_end(): Promise<Not> {
        const buf = Buffer.allocUnsafeSlow(16)
        buf.writeBigInt64BE(BigInt(this.free_start), 0)
        buf.writeBigInt64BE(BigInt(this.free_end), 8)
        await this.file.write(buf, 0)
    }
    
    /**
     * 读取分片
     * @param index 链表索引头
     */
    public async read(index: bigint) {
        
    }
}
