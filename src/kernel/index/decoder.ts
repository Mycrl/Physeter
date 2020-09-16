import { PrivateIndex } from "./index"

/**
 * 解码器
 * @param chunk 数据
 */
export default function(chunk: Buffer): PrivateIndex | null {
    let offset = 2

    // 检查长度
    if (chunk.length !== 66) {
        return null
    }

    // 检查固定位
    if (chunk.readUInt16BE(0) !== 0x9900) {
        return null
    }

    // 获取HASH
    let key = Buffer.allocUnsafeSlow(32)
    chunk.copy(key, 0, offset, offset + 32)
    offset += 32

    // 获取文件信息索引
    const metadata_start_track = Number(chunk.readBigInt64BE(offset))
    const metadata_start_index = Number(chunk.readBigInt64BE(offset + 8))
    offset += 16

    // 获取分片索引
    const chunk_start_track = Number(chunk.readBigInt64BE(offset))
    const chunk_start_index = Number(chunk.readBigInt64BE(offset + 8))

    return {
        start_chunk: [metadata_start_track, metadata_start_index],
        start_matedata: [chunk_start_track, chunk_start_index],
        key
    }
}
