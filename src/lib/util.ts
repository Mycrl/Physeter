import { createHash } from "crypto"

/**
 * 计算HASH
 * @param sign_source 源文本
 */
export function hash(sign_source: string): Buffer {
    const hash = createHash("sha256")
    hash.update(sign_source)
    return hash.digest()
}
