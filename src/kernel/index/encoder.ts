import { PrivateIndex } from "./index"

/**
 * 编码器
 * @param index 索引数据
 */
export default function(index: PrivateIndex) {
    let buffer = Buffer.allocUnsafeSlow(66)
    let offset = 0

    // 写入固定位
    buffer.writeUInt16BE(0x9900, offset)
    offset += 2

    // 写入HASH
    index.key.copy(buffer, offset)
    offset += 32

    // 写入文件信息轨道
    buffer.writeBigInt64BE(BigInt(index.start_matedata[0]), offset)
    offset += 8

    // 写入文件信息头索引
    buffer.writeBigInt64BE(BigInt(index.start_matedata[1]), offset)
    offset += 8

    // 写入分片轨道
    buffer.writeBigInt64BE(BigInt(index.start_chunk[0]), offset)
    offset += 8

    // 写入分片头索引
    buffer.writeBigInt64BE(BigInt(index.start_chunk[1]), offset)
    offset += 8

    return buffer
}
