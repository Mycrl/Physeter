import { KernelCompleteOptions } from "./kernel"
import File from "../lib/fs"
import { join } from "path"

/**
 * 轨道
 */
export interface Track {
    id: number                  // 轨道ID
    path: string                // 轨道文件路径
}

/**
 * 轨道类
 * @class
 */
export default class {
    private options: KernelCompleteOptions
    private file: File
    public path: string
    public id: number

    /**
     * @param options 核心配置
     * @constructor
     */
    constructor(id: number, options: KernelCompleteOptions) {
        this.path = join(options.directory, `${id}.track`)
        this.file = new File(this.path)
        this.options = options
        this.id = id
    }
    
    /**
     * 初始化
     * @desc 
     * !!! 外部需要强制调用初始化
     */
    public async initialize() {
        await this.file.initialize()
    }
}
