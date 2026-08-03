//! 位流读取器 — Fabian Giesen Variant 4 lookahead buffer。
//!
//! 参考：nickbabcock/bitter (multi-GiB/s), Fabian Giesen "Reading bits in far too many ways"

use std::cmp;

/// D2R 位流读取器（LSB-first per byte），lookahead u64 缓存。
#[derive(Debug, Clone)]
pub struct BitReader {
    data: Vec<u8>,
    byte_pos: usize,
    buf: u64,
    bits: u32,
    consumed: usize,
}

impl BitReader {
    #[inline]
    pub fn new(data: &[u8]) -> Self {
        Self { data: data.to_vec(), byte_pos: 0, buf: 0, bits: 0, consumed: 0 }
    }

    fn total(&self) -> usize { self.data.len() * 8 }

    #[inline] pub fn len_bits(&self) -> usize { self.total() }
    #[inline] pub fn is_empty(&self) -> bool { self.consumed >= self.total() }
    #[inline] pub fn remaining_bits(&self) -> usize { self.total() - cmp::min(self.consumed, self.total()) }
    #[inline] pub fn has_more(&self) -> bool { self.consumed < self.total() }
    #[inline] pub fn offset(&self) -> usize { self.consumed }

    pub fn seek(&mut self, pos: usize) {
        let clamped = cmp::min(pos, self.total());
        self.byte_pos = clamped / 8;
        self.buf = 0; self.bits = 0; self.consumed = clamped;
        let bit_off = clamped & 7;
        if bit_off != 0 && self.byte_pos < self.data.len() {
            let b = self.data[self.byte_pos] as u64;
            self.buf = b >> bit_off;
            self.bits = (8 - bit_off) as u32;
            self.byte_pos += 1;
        }
    }

    pub fn skip_bits(&mut self, n: usize) {
        let n = cmp::min(n, self.remaining_bits());
        let from_buf = cmp::min(n, self.bits as usize);
        self.buf >>= from_buf; self.bits -= from_buf as u32;
        self.consumed += from_buf;
        let remaining = n - from_buf;
        if remaining > 0 {
            self.byte_pos += remaining / 8;
            let bit_skip = remaining % 8;
            self.buf = 0; self.bits = 0;
            self.consumed += remaining;
            if bit_skip > 0 {
                self.refill_internal();
                let take = cmp::min(bit_skip, self.bits as usize);
                self.buf >>= take; self.bits -= take as u32;
            }
        }
    }

    #[inline] pub fn align(&mut self) {
        let off = self.consumed & 7;
        if off != 0 { self.skip_bits(8 - off); }
    }
    #[inline] pub fn align_to_byte(&mut self) { self.align(); }

    // ── 读取（always advances consumed by exactly n） ───

    #[inline]
    pub fn read_bit(&mut self) -> u8 {
        if self.consumed >= self.total() { self.consumed += 1; return 0; }
        if self.bits < 1 { self.refill_internal(); }
        let val = if self.bits >= 1 { (self.buf & 1) as u8 } else { 0 };
        if self.bits >= 1 { self.buf >>= 1; self.bits -= 1; }
        self.consumed += 1;
        val
    }

    #[inline]
    pub fn read_u8(&mut self, n: u8) -> u8 {
        debug_assert!(n <= 8);
        if n == 0 { return 0; }
        if self.bits < n as u32 { self.refill_internal(); }
        let actual = cmp::min(n, self.bits as u8);
        let val = if actual > 0 { (self.buf & ((1u64 << actual) - 1)) as u8 } else { 0 };
        if actual > 0 { self.buf >>= actual; self.bits -= actual as u32; }
        self.consumed += n as usize;
        val
    }

    #[inline]
    pub fn read_u16(&mut self, n: u8) -> u16 {
        debug_assert!(n <= 16);
        if n == 0 { return 0; }
        if self.bits < n as u32 { self.refill_internal(); }
        let actual = cmp::min(n, self.bits as u8);
        let val = if actual > 0 { (self.buf & ((1u64 << actual) - 1)) as u16 } else { 0 };
        if actual > 0 { self.buf >>= actual; self.bits -= actual as u32; }
        self.consumed += n as usize;
        val
    }

    #[inline]
    pub fn read_u32(&mut self, n: u8) -> u32 {
        debug_assert!(n <= 32);
        if n == 0 { return 0; }
        if self.bits < n as u32 { self.refill_internal(); }
        let actual = cmp::min(n, self.bits as u8);
        let val = if actual > 0 { (self.buf & ((1u64 << actual) - 1)) as u32 } else { 0 };
        if actual > 0 { self.buf >>= actual; self.bits -= actual as u32; }
        self.consumed += n as usize;
        val
    }

    pub fn read_string(&mut self, n: usize) -> String {
        (0..n).map(|_| self.read_u8(8) as char).collect()
    }

    pub fn read_bit_array(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.read_bit()).collect()
    }

    #[inline]
    pub fn peek_bits(&self, n: u8) -> u32 {
        debug_assert!(n <= 32);
        if n == 0 { return 0; }
        if self.bits >= n as u32 {
            (self.buf & ((1u64 << n) - 1)) as u32
        } else {
            self.peek_fallback(n)
        }
    }

    fn peek_fallback(&self, n: u8) -> u32 {
        let byte_start = self.consumed / 8;
        let bit_off = (self.consumed & 7) as u32;
        let mut val = 0u32;
        for i in 0..n as u32 {
            let byte_idx = byte_start + ((bit_off + i) >> 3) as usize;
            if byte_idx < self.data.len() {
                let b = self.data[byte_idx];
                let bit = (b >> ((bit_off + i) & 7)) & 1;
                val |= (bit as u32) << i;
            }
        }
        val
    }

    pub fn slice_to(&self, end_bit: usize) -> &[u8] {
        let start = self.consumed / 8;
        let end = cmp::min(end_bit.div_ceil(8), self.data.len());
        &self.data[cmp::min(start, end)..end]
    }

    fn refill_internal(&mut self) {
        let avail = self.data.len() - self.byte_pos;
        if avail == 0 { return; }
        let room = 64 - self.bits;
        let want = cmp::min(room, avail as u32 * 8);
        if want == 0 { return; }
        let take = (want / 8) as usize;
        if take == 0 { return; }
        for i in 0..take {
            self.buf |= (self.data[self.byte_pos + i] as u64) << (self.bits + i as u32 * 8);
        }
        self.byte_pos += take;
        self.bits += (take * 8) as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_read_u8_4bits() {
        let mut r = BitReader::new(&[0x16u8]);
        assert_eq!(r.read_u8(4), 0b0110); assert_eq!(r.consumed, 4);
    }
    #[test] fn test_read_bit() {
        let mut r = BitReader::new(&[0b10101010u8]);
        assert_eq!(r.read_bit(), 0); assert_eq!(r.read_bit(), 1);
    }
    #[test] fn test_align() {
        let mut r = BitReader::new(&[0xFF, 0xFF]);
        r.skip_bits(3); r.align(); assert_eq!(r.consumed, 8);
    }
    #[test] fn test_peek_bits_does_not_advance() {
        let mut r = BitReader::new(&[0b10110100u8]);
        assert_eq!(r.peek_bits(4), 0b0100); assert_eq!(r.consumed, 0);
        assert_eq!(r.read_u8(4), 0b0100);
    }
    #[test] fn test_eof_advances_and_stays_consistent() {
        let mut r = BitReader::new(&[0u8; 1]);
        r.read_u32(8); // reads 8 bits
        assert_eq!(r.read_u8(4), 0); // 4 more but only 0 avail → returns 0, consumed=8+4=12
        assert_eq!(r.consumed, 12);
        assert!(r.is_empty());
        assert_eq!(r.remaining_bits(), 0);
    }
    #[test] fn test_read_u32_partial_end() {
        let mut r = BitReader::new(&[0xFF]);
        assert_eq!(r.read_u32(32), 0xFF); // only 8 bits real, rest 0
        assert_eq!(r.consumed, 32);
    }
    #[test] fn test_remaining_bits_saturates() {
        let mut r = BitReader::new(&[0; 1]);
        r.skip_bits(100); // clamped to 8
        assert_eq!(r.consumed, 8);
        assert_eq!(r.remaining_bits(), 0);
        assert!(r.is_empty());
    }
}