import { Kernel, KernelCompleteOptions } from "./kernel"
import SuperEvents from "../lib/super_events"
import { exists } from "../lib/fs"
import { Track } from "./track"
import { join } from "path"

/**
 * 默认核心数据
 * @param track 初始轨道
 * @desc 轨道列表和索引列表全为空
 */
async function defaultKernelData(track: Track): Promise<Kernel> {
    await track.initialize()
    const track_list = [track]
    return { track_list, index_list: [] }
}

/**
 * 存储类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private events: SuperEvents<any, any>

    /**
     * @param options 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.events = new SuperEvents()
        this.options = options
    }

    /**
     * 默认核心
     * @desc
     * 当索引文件不存在时创建默认索引数据，
     * 同时初始化轨道文件.
     */
    private async default_kernel(): Promise<Kernel> {
        const track = new Track(0, this.events, this.options)
        return await defaultKernelData(track)
    }

    /**
     * 初始化
     */
    public async initialize() {
        const { directory } = this.options
        const index_filename = join(directory, "index")

        /**
         * 如果索引文件不存在
         * 则返回空索引
         * 并创建初始轨道
         */
        if (!await exists(index_filename)) {
            return await this.default_kernel()   
        }
    }
}
