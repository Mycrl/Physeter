import { Not } from "./util"
import fs from "fs"

/**
 * 创建写入流
 * @param path 路径
 */
export function createWriteStream(path: string): fs.WriteStream {
    return fs.createWriteStream(path)
}

/**
 * 创建读取流
 * @param path 路径
 */
export function createReadStream(path: string): fs.ReadStream {
    return fs.createReadStream(path)
}

/**
 * 检查路径是否存在
 * @param path 路径
 */
export function exists(path: string): Promise<boolean> {
    return new Promise(resolve => fs.open(path, "r", (err, fd) => {
        err ? resolve(false) : fs.close(fd, err => {
            resolve(err ? false : true)
        })
    }))
}

/**
 * 读取目录
 * @param path 路径
 */
export function readdir(path: string): Promise<string[]> {
    return new Promise((resolve, reject) => fs.readdir(path, (err, files) => {
        err ? reject(err) : resolve(files)
    }))
}

/**
 * 写入文件
 * @param path 路径
 * @param data 数据
 */
export function writeFile(path: string, data: Buffer | string): Promise<Not> {
    return new Promise((resolve, reject) => fs.writeFile(path, data, err => {
        err ? reject(err) : resolve()
    }))
}


/**
 * 写入文件句柄
 * @param path 路径
 * @param data 数据
 */
export function write(fd: number, data: Buffer, offset: number, length: number, position: number): Promise<number> {
    return new Promise((resolve, reject) => fs.write(fd, data, offset, length, position, (err, size) => {
        err ? reject(err) : resolve(size)
    }))
}

/**
 * 读取文件句柄
 * @param path 路径
 * @param data 数据
 */
export function read(fd: number, data: Buffer, offset: number, length: number, position: number): Promise<number> {
    return new Promise((resolve, reject) => fs.read(fd, data, offset, length, position, (err, size) => {
        err ? reject(err) : resolve(size)
    }))
}

/**
 * 创建或者打开
 * @param path 路径
 */
export function createAndOpen(path: string): Promise<number> {
    return new Promise(async (resolve, reject) => {
        if (!await exists(path)) await writeFile(path, "")
        fs.open(path, "r+", (err, fd) => {
            err ? reject(err) : resolve(fd)
        })
    })
}

/**
 * 全部分配
 * @desc 用于完整写入
 */
type Handle = (...argv: number[]) => Promise<number>
async function alloc_all(count_size: number, offset: number, handle: Handle, buf_offset = 0): Promise<Not> {
    for (;;) {
        const position = offset + buf_offset
        const length = count_size - buf_offset
        const size = await handle(buf_offset, length, position)
        if (buf_offset + size >= count_size) break
        buf_offset += size
    }
}

/**
 * 文件类
 * @class
 */
export default class {
    private path: string
    private fd?: number

    /**
     * @param path 地址
     * @constructor
     */
    constructor(path: string) {
        this.path = path
    }

    /**
     * 初始化
     * @desc 
     * !!! 外部需要强制调用初始化
     */
    public async initialize(): Promise<Not> {
        this.fd = await createAndOpen(this.path)
    }

    // 创建写入流
    public createWriteStream(): fs.WriteStream {
        return fs.createWriteStream(this.path, { fd: this.fd! })
    }

    // 创建读取流
    public createReadStream(): fs.ReadStream {
        return fs.createReadStream(this.path, { fd: this.fd! })
    }
  
    // 获取文件信息
    public stat(): Promise<fs.Stats> {
        return new Promise((resolve, reject) => {
            fs.stat(this.path, (err, stats) => {
                err ? reject(err) : resolve(stats)
            })
        })
    }

    /**
     * 追加数据到文件
     * @param chunk 数据
     */
    public append(chunk: Buffer): Promise<Not> {
        return new Promise((resolve, reject) => {
            fs.appendFile(this.path, chunk, err => {
                err ? reject(err) : resolve()
            })
        })
    }

    /**
     * 写入数据
     * @desc 这个接口会强制完成
     * @param chunk 数据
     */
    public async write(chunk: Buffer, offset: number): Promise<Not> {
        await alloc_all(chunk.length, offset, async (buf_offset, length, position) => {
            return await write(this.fd!, chunk, buf_offset, length, position)
        })
    }

    /**
     * 读取数据
     * @desc 这个接口会强制完成
     * @param chunk 数据
     */
    public async read(chunk: Buffer, offset: number): Promise<number> {
        return await await read(this.fd!, chunk, 0, chunk.length, offset)
    }
}
