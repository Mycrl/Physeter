import { Kernel, KernelCompleteOptions } from "./kernel"
import { exists } from "../lib/fs"
import Track from "./track"
import { join } from "path"

/**
 * 默认核心数据
 * @desc 轨道列表和索引列表全为空
 */
function defaultKernelData(): Kernel {
    return {
        track_list: [],
        index_list: []
    }
}

/**
 * 存储类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    
    /**
     * @param {options} 核心配置
     * @constructor
     */
    constructor(options: KernelCompleteOptions) {
        this.options = options
    }
    
    /**
     * 默认核心
     * @desc
     * 当索引文件不存在时创建默认索引数据，
     * 同时初始化轨道文件.
     */
    private default_kernel(): Kernel {
        let kernel_data = defaultKernelData()
        kernel_data.track_list.push(new Track(0, this.options))
        return kernel_data
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
        if (!await exists(index_filename))
            return this.default_kernel()
    }
}