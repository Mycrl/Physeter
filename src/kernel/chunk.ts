/**
 * 分片
 */
export interface Chunk {
    id: number                  // 分片ID
    exist: boolean              // 分片是否有效
    next?: number               // 下个分片索引
    next_track?: number         // 下个分片轨道
    data: Buffer                // 数据
}