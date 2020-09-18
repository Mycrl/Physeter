import { createHash } from "crypto"

/**
 * 为了解决编译器无法识别void的问题
 * 重命名void类型
 */
export type Not = void

/**
 * 计算HASH
 * @param sign_source 源文本
 */
export function hash(sign_source: string): Buffer {
    const hash = createHash("sha256")
    hash.update(sign_source)
    return hash.digest()
}