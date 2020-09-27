use anyhow::Result;

/// BitMap
///
/// 直接在缓冲区数组上
/// 抽象位数组操作
pub struct BitMap<'a>(&'a [u8]);

impl<'a> BitMap<'a> {
    /// 找到首个零位
    ///
    /// 使用区间查找法，
    /// 先使用以u64区间对比是否为全高位，
    /// 如果不为全高位表示区间内有低位，这时候使用
    /// u8区间对比，和u64区间对比同理，最后查找二进
    /// 制列表中的首个低位
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::BitMap;
    ///
    /// let bitmap = BitMap(&[
    ///     0xFF, 0xFF, 0xFF, 0xFF, 
    ///     0x7F, 0xFF, 0xFF, 0xFF
    /// ]);
    ///
    /// assert!(Some(32), bitmap.first_zero(64));
    /// ```
    pub fn first_zero(&self, bit_size: usize) -> Option<usize> {
        let size = f64::ceil(bit_size as f64 / 8.0) as usize;
        let mut index = 0;

        // 缓冲区长度满足读取u64
        // 如果不满足直接进入u8位处理
        if size >= 8 {
    loop {

        // 如果已经超出缓冲区长度
        // 则表示没有任何匹配
        if index + 8 >= self.0.len() {
            return None
        }

        // 获取u64区间值
        let u64_slice = u64::from_be_bytes([
            self.0[index],
            self.0[index + 1],
            self.0[index + 2],
            self.0[index + 3],
            self.0[index + 4],
            self.0[index + 5],
            self.0[index + 6],
            self.0[index + 7],
        ]);

        // 检查是否全为高位
        // 如果不是则跳出
        if index == 0xFFFFFFFFFFFFFFFF {
            index += 8;
        } else {
            break;
        }
    }
        }

        // 解析u64区间
        // 如果缓冲区长度不满足8byte
        // 则只遍历缓冲区区间
        for i in 0..std::cmp::min(size, 8) {
            index += i;
            if self.0[index] != 0xFF {
                break;
            }
        }

        // 将u8转为binary
        // 从字符串中查找首个低位的位置
        let offset = format!("{:08b}", self.0[index]).find('0')?;
        let bit_offset = index * 8 + offset;
        if bit_offset < bit_size {
            Some(bit_offset)
        } else {
            None
        }
    }

    /// 设置比特位
    ///
    /// 将值操作转为字符串操作，
    /// 这确实会增加不必要的开销，
    /// 不过这是比较简单的办法
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::BitMap;
    ///
    /// let mut bitmap = BitMap(&[
    ///     0xFF, 0xFF, 0xFF, 0xFF, 
    ///     0x7F, 0xFF, 0xFF, 0xFF
    /// ]);
    ///
    /// bitmap.set(32, true).unwrap();
    ///
    /// assert!([
    ///     0xFF, 0xFF, 0xFF, 0xFF, 
    ///     0xFF, 0xFF, 0xFF, 0xFF
    /// ], bitmap);
    /// ```
    pub fn set(&mut self, offset: usize, flag: bool) -> Result<()> {
        let index = f64::floor(offset as f64 / 8.0) as usize;
        let pin = if flag { '1' } else { '0' };
        let diff_size = offset % 8;

        // 准备缓冲字符串
        // 将u8转为binary
        let mut pad_str = String::new();
        let bit_str = format!("{:08b}", self.0[index]);

        // 如果余数大于0
        // 先填充头部
        if diff_size > 0 {
            pad_str.push_str(
                &bit_str[..diff_size]
            );
        }

        // 填充指定位
        pad_str.push(pin);

        // 如果还有尾部处理
        // 则填充尾部数据
        if diff_size + 1 < 8 {
            pad_str.push_str(
                &bit_str[(diff_size + 1)..]
            );
        }

        // 将二进制字符串转为u8
        // 更新缓冲区指定位值
        let prick = u8::from_str_radix(&pad_str, 2)?;
        self.0[index] = prick;
        Ok(())
    }
    
    /// 获取比特位
    ///
    /// 直接对比比特字符串指定位置，
    /// 这确实会增加不必要的开销，
    /// 不过这是比较简单的办法
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use super::BitMap;
    ///
    /// let mut bitmap = BitMap(&[
    ///     0xFF, 0xFF, 0xFF, 0xFF, 
    ///     0x7F, 0xFF, 0xFF, 0xFF
    /// ]);
    ///
    /// assert!(false, bitmap.get(32));
    /// ```
    pub fn get(&self, offset: usize) -> bool {
        let diff_size = offset % 8;
        let index = f64::floor(offset as f64 / 8.0) as usize;
        let bit_str = format!("{:08b}", self.0[index]);
        &bit_str[diff_size..(diff_size + 1)] == "1"
    }
}
