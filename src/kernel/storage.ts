import { KernelCompleteOptions } from "./kernel"
import { Readable, Writable } from "stream"
import { exists, readdir } from "../lib/fs"
import { Not } from "../lib/util"
import { Track } from "./track"
import { join } from "path"

/**
 * 存储类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private tracks: { [key: number]: Track }

    /**
     * @param options 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.options = options
        this.tracks = {}
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
            this.tracks[track_id] = new Track(track_id, this.options)
            await this.tracks[track_id].initialize()
        }
    }
    
    /**
     * 读取数据
     * @param index 链表头索引
     */
    public async read(index: bigint) {
        
    }
}

/**
 * 可读流
 * @class
 */
class Reader extends Readable {
    private track: number
    private index: bigint
    
    /**
     * @param track 初始轨道
     * @param index 链表头索引
     * @constructor
     */
    constructor(track: number, index: bigint) {
        super()
        this.track = track
        this.index = index
    }
    
    /**
     * 读取流
     * @param size 读取长度
     */
    async _read(size: number) {
        
    }
    
    /**
     * 销毁流
     */
    _destroy() {
        
    }
}
