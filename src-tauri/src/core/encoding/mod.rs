//! 通用 Huffman 编码/解码。
//!
//! D2R item 类型字符串使用 4 字符 Huffman 编码，
//! 来源于游戏 webpack chunk 的硬编码表 `HUFFMAN_LOOKUP`。
//!
//! 字符路径用 LSB-first 描述：`HUFFMAN_LOOKUP` 中 `value` 的第 b 位
//! 即为树上 b 步的方向（0=左，1=右）。
//!
//! ## 性能优化
//! 解码从树遍历改为 512-entry 查表（max code length = 9 bits）。
//! 单次解码从 ~30 次节点访问 + 分支降至 1 次 peek(9) + 1 次查表 + 1 次 skip_bits。

use crate::core::bitio::{BitReader, BitWriter};
use std::sync::OnceLock;

/// Huffman 字符编码表（character, value, bit_length）。
/// `value` 在 LSB-first 下描述 Huffman 树路径。
pub const HUFFMAN_LOOKUP: &[(char, u16, u8)] = &[
    ('0', 223, 8), ('1', 31, 7), ('2', 12, 6), ('3', 91, 7),
    ('4', 95, 8), ('5', 104, 8), ('6', 123, 7), ('7', 30, 5),
    ('8', 8, 6), ('9', 14, 5), (' ', 1, 2), ('a', 15, 5),
    ('b', 10, 4), ('c', 2, 5), ('d', 35, 6), ('e', 3, 6),
    ('f', 50, 6), ('g', 11, 5), ('h', 24, 5), ('i', 63, 7),
    ('j', 232, 9), ('k', 18, 6), ('l', 23, 5), ('m', 22, 5),
    ('n', 44, 6), ('o', 127, 7), ('p', 19, 5), ('q', 155, 8),
    ('r', 7, 5), ('s', 4, 4), ('t', 6, 5), ('u', 16, 5),
    ('v', 59, 7), ('w', 0, 5), ('x', 28, 5), ('y', 40, 7),
    ('z', 27, 8),
];

/// 最大 Huffman 码长（bits）。
const MAX_CODE_BITS: u8 = 9;

/// Lookup table 的单条目。
#[derive(Debug, Clone, Copy)]
struct HuffmanEntry {
    /// 解码出的字符（ASCII 字节）。
    ch: u8,
    /// 码长（bits），用于 skip_bits。
    len: u8,
}

/// 构建 Huffman 解码查找表（512 entries = 2^9）。
///
/// LSB-first 编码：peek_bits(N) 的低 `bits` 位即为 code `value`，
/// 高位为 don't-care。所以表索引 = `value | (don't_care << bits)`。
fn build_huffman_table() -> [HuffmanEntry; 1 << MAX_CODE_BITS] {
    let mut table = [HuffmanEntry { ch: b'?', len: 0 }; 1 << MAX_CODE_BITS];

    for &(ch, value, bits) in HUFFMAN_LOOKUP {
        let shift = MAX_CODE_BITS - bits;
        let entry = HuffmanEntry {
            ch: ch as u8,
            len: bits,
        };

        // LSB-first: code 占低位，don't-care 占高位
        // 所有 PEEK = value | (higher_bits << bits) 的表项都映射到此字符
        for i in 0..(1usize << shift) {
            table[value as usize | (i << bits)] = entry;
        }
    }

    table
}
/// 获取懒初始化的解码查找表。
fn get_huffman_table() -> &'static [HuffmanEntry; 1 << MAX_CODE_BITS] {
    static TABLE: OnceLock<[HuffmanEntry; 1 << MAX_CODE_BITS]> = OnceLock::new();
    TABLE.get_or_init(build_huffman_table)
}

/// 解码一个 4-char 物品类型字符串（查表版本）。
///
/// peek(9) 一次 -> 查表得字符和码长 -> skip_bits(码长)。
/// 重复 4 次，最后 trim 尾部空格。
pub fn decode_huffman_string(reader: &mut BitReader) -> String {
    let table = get_huffman_table();
    let mut buf = [0u8; 4];

    for slot in buf.iter_mut() {
        let peek = reader.peek_bits(MAX_CODE_BITS);
        let entry = table[peek as usize];
        *slot = entry.ch;
        reader.skip_bits(entry.len as usize);
    }

    // 与原始行为一致：读满 4 字符后 trim
    String::from_utf8_lossy(&buf).trim().to_string()
}

/// Encode a 4-character item type string using Huffman lookup table.
/// Writes `value` as `bits` bits, LSB first.
pub fn encode_huffman_string(writer: &mut BitWriter, s: &str) {
    let padded = format!("{:<4}", s);
    for c in padded.chars() {
        encode_huffman_char(writer, c);
    }
}

/// Encode a single character using the Huffman lookup table.
pub fn encode_huffman_char(writer: &mut BitWriter, c: char) {
    if let Some(&(_, value, bits)) = HUFFMAN_LOOKUP.iter().find(|(ch, _, _)| *ch == c) {
        writer.write_u16(value, bits);
    } else {
        // Fallback: encode space (value=1, 2 bits)
        writer.write_u16(1, 2);
    }
}

/// Skip a D2R 7-bit ASCII string (terminated when MSB of the 7-bit char is set).
/// Used for character names, personalization, and ear items.
pub fn skip_string_7bit(reader: &mut BitReader) {
    loop {
        if reader.remaining_bits() < 7 {
            break;
        }
        let c = reader.read_u8(7);
        if c == 0 || c > 0x7E {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_has_all_chars() {
        let table = build_huffman_table();
        // Verify all characters are reachable from the table
        for &(ch, _, _) in HUFFMAN_LOOKUP {
            let found = table.iter().any(|e| e.ch == ch as u8);
            assert!(found, "Character '{}' not found in decode table", ch);
        }
    }

    #[test]
    fn test_huffman_all_chars_roundtrip() {
        for &(ch, _, _) in HUFFMAN_LOOKUP {
            // Encode
            let mut writer = BitWriter::new(16);
            encode_huffman_char(&mut writer, ch);
            let bytes = writer.to_bytes();

            // Decode
            let mut reader = BitReader::new(&bytes);
            let peek = reader.peek_bits(MAX_CODE_BITS);
            let entry = get_huffman_table()[peek as usize];
            let decoded = entry.ch as char;
            // Need to also skip the bits
            reader.skip_bits(entry.len as usize);
            assert_eq!(decoded, ch, "Round-trip failed for char '{}'", ch);
        }
    }

    #[test]
    fn test_huffman_string_roundtrip() {
        let test_cases = vec!["r01", "gcv", "rvs", "pk1", "toa", "tes", "skz", "gpv"];
        for input in &test_cases {
            let mut writer = BitWriter::new(64);
            encode_huffman_string(&mut writer, input);
            let bytes = writer.to_bytes();

            let mut reader = BitReader::new(&bytes);
            let decoded = decode_huffman_string(&mut reader);
            assert_eq!(decoded, *input, "Round-trip failed for '{}'", input);
        }
    }

    #[test]
    fn test_space_roundtrip() {
        let mut writer = BitWriter::new(16);
        encode_huffman_char(&mut writer, ' ');
        let bytes = writer.to_bytes();

        let mut reader = BitReader::new(&bytes);
        // decode_huffman_string 始终读 4 字符后 trim。
        // 单个 space 编码后 2 位，后续不足的位解码为 'w'。
        // space is first char + 3 'w' fillers → " www" → trim → "www"
        let decoded = decode_huffman_string(&mut reader);
        assert_eq!(decoded, "www", "single space -> trim leading space -> 'www'");
    }
#[allow(dead_code)]
    fn test_decode_huffman_string_produces_3char_code() {
        // "r01" encoded
        let mut writer = BitWriter::new(64);
        encode_huffman_string(&mut writer, "r01");
        let bytes = writer.to_bytes();

        let mut reader = BitReader::new(&bytes);
        let result = decode_huffman_string(&mut reader);
        assert_eq!(result, "r01");
    }
}