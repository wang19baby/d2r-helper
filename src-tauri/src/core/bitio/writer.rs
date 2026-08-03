//! 通用位流写入器（LSB-first per byte）。
//!
//! 使用 `Vec<u8>` + 字节内位偏移实现，已移除 bitvec 依赖。

#[derive(Debug, Clone)]
pub struct BitWriter {
    bytes: Vec<u8>,
    bit_len: usize, // 总写入位数
}

impl BitWriter {
    /// 创建写入器，`capacity_bits` 是预分配的位容量。
    pub fn new(capacity_bits: usize) -> Self {
        let byte_cap = capacity_bits.div_ceil(8);
        Self {
            bytes: Vec::with_capacity(byte_cap.max(8)),
            bit_len: 0,
        }
    }

    /// 写入 1 位。
    #[inline]
    pub fn write_bit(&mut self, bit: u8) {
        let byte_idx = self.bit_len >> 3;
        let bit_off = self.bit_len & 7;
        if byte_idx >= self.bytes.len() {
            self.bytes.push(0);
        }
        if bit != 0 {
            self.bytes[byte_idx] |= 1 << bit_off;
        }
        self.bit_len += 1;
    }

    /// 写入 n 位（从 `value` 的低 n 位取，LSB-first）。
    #[inline]
    pub fn write_u8(&mut self, value: u8, n: u8) {
        for i in 0..n {
            self.write_bit((value >> i) & 1);
        }
    }

    /// 写入 n 位 u16。
    #[inline]
    pub fn write_u16(&mut self, value: u16, n: u8) {
        for i in 0..n {
            self.write_bit(((value >> i) & 1) as u8);
        }
    }

    /// 写入 n 位 u32。
    #[inline]
    pub fn write_u32(&mut self, value: u32, n: u8) {
        for i in 0..n {
            self.write_bit(((value >> i) & 1) as u8);
        }
    }

    /// 写入字节数组（每字节 8 位）。
    pub fn write_bytes(&mut self, data: &[u8]) {
        for &b in data {
            self.write_u8(b, 8);
        }
    }

    /// 写入字符串为 n 字节（UTF-8）。
    pub fn write_string(&mut self, s: &str, n: usize) {
        let bytes = s.as_bytes();
        for i in 0..n {
            let b = bytes.get(i).copied().unwrap_or(0);
            self.write_u8(b, 8);
        }
    }

    /// 写入位数组（每元素 0/1）。
    pub fn write_bits(&mut self, bits: &[u8]) {
        for &b in bits {
            self.write_bit(b);
        }
    }

    /// 对齐到字节边界（补 0）。
    pub fn align(&mut self) {
        let off = self.bit_len & 7;
        if off != 0 {
            for _ in 0..(8 - off) {
                self.write_bit(0);
            }
        }
    }

    /// 当前长度（位）。
    pub fn len_bits(&self) -> usize {
        self.bit_len
    }

    /// 转为字节数组。
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.bytes.clone();
        // 截断多余的预分配字节
        let needed = self.bit_len.div_ceil(8);
        result.truncate(needed);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_u8() {
        let mut writer = BitWriter::new(16);
        writer.write_u8(0b0110, 4);
        assert_eq!(writer.len_bits(), 4);
        let bytes = writer.to_bytes();
        assert_eq!(bytes.len(), 1);
        assert_eq!(bytes[0], 0b00000110);
    }

    #[test]
    fn test_write_bytes() {
        let original = [0xABu8, 0xCD, 0xEF];
        let mut writer = BitWriter::new(24);
        writer.write_bytes(&original);
        let result = writer.to_bytes();
        assert_eq!(&original[..], &result[..result.len().min(original.len())]);
    }

    #[test]
    fn test_align() {
        let mut writer = BitWriter::new(8);
        writer.write_u8(0x0F, 4);
        assert_eq!(writer.len_bits(), 4);
        writer.align();
        assert_eq!(writer.len_bits(), 8);
    }

    #[test]
    fn test_write_read_roundtrip() {
        let mut w = BitWriter::new(64);
        w.write_u8(0b1010, 4);
        w.write_u16(0x1234, 16);
        w.write_bit(1);
        w.write_string("JM", 2);
        let bytes = w.to_bytes();

        let mut r = crate::core::BitReader::new(&bytes);
        assert_eq!(r.read_u8(4), 0b1010);
        assert_eq!(r.read_u16(16), 0x1234);
        assert_eq!(r.read_bit(), 1);
        assert_eq!(r.read_string(2), "JM");
    }
}