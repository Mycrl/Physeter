use bytes::{Buf, BufMut, BytesMut, Bytes};

/// 解码失效列表
pub fn decoder(mut chunk: BytesMut) -> Vec<u64> {
    let first = chunk.get_u16() as usize;
    let size = chunk.get_u16() as usize;
    let mut index = Vec::new();

    if first > 0 {
        chunk.advance(first);
    }

    for _ in 0..(size / 8) {
        index.push(chunk.get_u64());
    }

    index
}

/// 编码失效列表
pub fn encoder(index: Vec<u64>) -> Bytes {
    let mut packet = BytesMut::new();
    
    packet.put_u16(0);
    packet.put_u16((index.len() * 8) as u16);

    for v in index {
        packet.put_u64(v);
    }

    packet.freeze()
}

/// 消费失效块
pub fn consume(chunk: &mut [u8]) {
    let mut first = u16::from_be_bytes([
        chunk[0],
        chunk[1]
    ]);

    let mut size = u16::from_be_bytes([
        chunk[2],
        chunk[3]
    ]);

    first += 8;
    size -= 8;

    let first_buf = first.to_be_bytes();
    let size_buf = size.to_be_bytes();

    chunk[0] = first_buf[0];
    chunk[1] = first_buf[1];

    chunk[2] = size_buf[0];
    chunk[3] = size_buf[1];
}

/// 添加时效块
pub fn append(chunk: &mut [u8], value: u64) {
    let value_buf = value.to_be_bytes();

    let mut first = u16::from_be_bytes([
        chunk[0],
        chunk[1]
    ]);

    let mut size = u16::from_be_bytes([
        chunk[2],
        chunk[3]
    ]);

    size += 8;

    let size_buf = size.to_be_bytes();
    chunk[2] = size_buf[0];
    chunk[3] = size_buf[1];

    let index = (first + size + 4) as usize;
    
    chunk[index] = value_buf[0];
    chunk[index + 1] = value_buf[1];
    chunk[index + 2] = value_buf[2];
    chunk[index + 3] = value_buf[3];
    chunk[index + 4] = value_buf[4];
    chunk[index + 5] = value_buf[5];
    chunk[index + 6] = value_buf[6];
    chunk[index + 7] = value_buf[7];
}