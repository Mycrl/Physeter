import fs from "fs"

/**
 * 创建写入流
 * @param {string} path 路径
 */
export function createWriteStream(path: string) {
    return fs.createWriteStream(path)
}

/**
 * 检查路径是否存在
 * @param {string} path 路径
 */
export function exists(path: string): Promise<boolean> {
    return new Promise(resolve => {
        fs.open(path, "r", (err, fd) => {
            err ? resolve(false) : fs.close(fd, err => {
                resolve(err ? false : true)
            })
        })
    })
}