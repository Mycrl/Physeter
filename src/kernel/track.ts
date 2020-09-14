import { KernelCompleteOptions } from "./kernel"
import { createWriteStream } from "../lib/fs"
import { join } from "path"

/**
 * 轨道
 */
export interface Track {
    id: number                  // 轨道ID
    path: string                // 轨道文件路径
}

/**
 * 轨道文件分配块大小
 * @desc 
 * 强调单次分配大小的目的是为了
 * 避免单次申请太多内存而导致堆内存溢出，
 * 所以控制单次申请大小而达到优化GC的目的.
 */
const TRACK_ALLOC_CHUNK_SIZE = 1024 * 1024 * 1024

/**
 * 轨道类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    public path: string
    public id: number
    
    /**
     * @param {options} 核心配置
     * @constructor
     */
    constructor(id: number, options: KernelCompleteOptions) {
        this.path = join(options.directory, id + ".track")
        this.options = options
        this.initialize()
        this.id = id
    }
    
    /**
     * 初始化轨道文件
     * @desc 
     * 创建轨道文件并预填充数据
     * 预填充的目的是因为文件系统写入数据时并不一定将文件分片按
     * 相邻位置写入，当读取或者写入的时候会因为受不稳定寻道的影响，
     * 所以这里首次先预先填充空数据，这样可以使文件内容的位置尽量靠拢.
     */
    public initialize() {
        const writable = createWriteStream(this.path)
        const { track_size } = this.options
        
        /**
         * 计算分配的次数
         * 计算分配溢出大小
         * 这里计算分配的次数为向上取整，抛弃余数，
         * 再次单独计算余数
         */
        const alloc_size = Math.floor(track_size / TRACK_ALLOC_CHUNK_SIZE)
        const end_pad = track_size % TRACK_ALLOC_CHUNK_SIZE
        
        // 按分配次数将空数据写入文件
        for (let i = 0; i < alloc_size; i ++)
            writable.write(Buffer.alloc(TRACK_ALLOC_CHUNK_SIZE))
        
        /**
         * 再次写入余下数据
         * 并关闭可写流
         */
        writable.end(Buffer.alloc(end_pad))
        writable.destroy()
    }
}