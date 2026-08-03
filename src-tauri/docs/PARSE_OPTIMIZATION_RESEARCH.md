# D2I 解析器优化方案调研

## 一、现状分析

### 1.1 性能数据（来自测试输出）

```
Page 5 (堆叠/符文): declared=131, found=109, recall=83.2%
  parse_time=7.8ms, scan_time=17.5ms, scan_count=75

Page 0 (非堆叠/复杂): declared=80, found=24, recall=30.0%
  parse_time=11.9ms, scan_time=624.2ms, scan_count=227

Page 1 (非堆叠): declared=66, found=12, recall=18.2%
  scan_time=800.1ms, scan_count=309
```

**核心问题**: scan_time 远大于 parse_time，非堆叠页面 forward-scan 开销是解析本身的 50-70 倍。

### 1.2 热点代码定位

#### 热点1: `BitReader` 逐位循环读取 (`core/bitio/reader.rs`)

```rust
// 当前实现: 读 32 bits = 32 次循环 + 32 次边界检查
pub fn read_u32(&mut self, n: u8) -> u32 {
    let mut value: u32 = 0;
    for i in 0..n as usize {
        if self.offset < self.bits.len() && self.bits[self.offset] {
            value |= 1u32.checked_shl(i as u32).unwrap_or(0);
        }
        self.offset += 1;
    }
    value
}
```

每个 item header 解析涉及:
- `read_u32(32)` flags → 32 次循环
- `read_u8(3)` version → 3 次循环
- `read_u8(4)` quality → 4 次循环
- `read_u8(7)` ilvl → 7 次循环
- `read_u32(32)` uid → 32 次循环
- 加上 body 内大量 read_u8/read_u32

**单个 item 解析可能有 200+ 次循环**，`parse_noncompact_body` 内 stat_list 解析尤其重。

#### 热点2: `scan_next_item` 每次创建新 BitReader (`jm_reader.rs:58-89`)

```rust
fn scan_next_item(payload: &[u8], start: usize, config: &ScanConfig) -> Option<ScanResult> {
    for skip in 1..=max_scan {  // max_scan=32
        let probe = start + skip * 8;
        
        // 问题1: 每次循环创建新 BitReader
        let mut pr = BitReader::new(payload);
        pr.seek(probe);
        let pf = pr.read_u32(32);  // 32 次循环
        let pv = pr.read_u8(3);     // 3 次循环
        
        // 问题2: 不满足条件时继续，再创建第2个 BitReader 读 code
        let mut cr = BitReader::new(payload);
        cr.seek(probe + 53);
        let code = decode_huffman_string(&mut cr);
    }
}
```

一次失败的 forward-scan: 最多 32 × 2 = **64 个 BitReader 创建**，每次创建都重新遍历 bitvec。max_scan=32，每次失败开销极大。

#### 热点3: `parse_noncompact_body` 复杂 stat list 解析 (`jm_reader.rs:207+`)

堆叠页面(简单物品) recall 83%，非堆叠页面(复杂装备) recall 仅 18-40%。stat list 解析对复杂物品失败率更高，导致更多 forward-scan。

---

## 二、优化方案

### 2.1 BitReader 批量位操作（高优先级）

**原理**: 不逐 bit 循环，而是每次读多个 byte，用掩码+移位提取 bits。

```rust
// 当前: O(n) 次循环 + 每次边界检查
// 优化后: O(n/32) 次读 + O(1) 位操作

impl BitReader {
    /// 批量读取 n bits (n <= 32)，用 2 次 u32 读 + 掩码
    pub fn read_u32_fast(&mut self, n: u8) -> u32 {
        if n == 0 { return 0; }
        let n = n as usize;
        
        // byte 对齐到当前 bit 位置
        let byte_offset = self.offset / 8;
        let bit_offset = self.offset % 8;
        
        // 最多需要 4 bytes (32 bits)
        let buf = &self.bits.as_raw_slice()[byte_offset..];
        if buf.len() < 4 {
            return self.read_u32_slow_path(n); // fallback 边界情况
        }
        
        let mut word = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        
        // 如果 bit_offset != 0，需要右移对齐
        if bit_offset != 0 {
            word >>= bit_offset;
        }
        
        self.offset += n;
        let mask = (1u32 << n) - 1;
        word & mask
    }
    
    // 对 read_u8, read_u16 也做类似优化
    pub fn read_u8_fast(&mut self, n: u8) -> u8 {
        if n == 0 { return 0; }
        let n = n as usize;
        let byte_offset = self.offset / 8;
        let bit_offset = self.offset % 8;
        
        let buf = self.bits.as_raw_slice();
        if byte_offset >= buf.len() { return 0; }
        
        let mut word = buf[byte_offset];
        if bit_offset != 0 && byte_offset + 1 < buf.len() {
            word |= buf[byte_offset + 1] << 8;
            word >>= bit_offset;
        }
        
        self.offset += n;
        let mask = (1u8 << n) - 1;
        word & mask
    }
}
```

**预期收益**: `read_u32` 从 32 次循环降到 1 次批量读，**提速 10-20 倍**。整个解析过程可能有 30-50% 的 CPU 时间在 BitReader 操作上。

**风险**: 低。`bitvec` 底层是 `Vec<u8>`，直接索引是安全的。只需要处理边界情况（bit 不对齐、剩余 bits < n）。

**bitvec 已有 SIMD**: `bitvec` 库的 `BitVec::load` 方法已经在内部做了 chunk 操作，可以直接利用:

```rust
// 检查 bitvec 是否支持
fn read_u32_via_bitvec(&mut self, n: u8) -> u32 {
    let n = n as usize;
    let start = self.offset;
    let end = start + n;
    
    // BitVec::load 读取整个 usize 块（64 bits on 64-bit arch）
    // 然后用掩码提取需要的 bits
    let word = self.bits[start..end].load::<u32>();
    self.offset = end;
    word
}
```

### 2.2 SIMD 并行 Forward-Scan（高优先级）

**原理**: 用 SIMD 指令并行扫描多个 byte，寻找 item header 模式。

```rust
// 当前: 逐 bit/byte 串行扫描
// SIMD: 一次检查 16/32/64 bytes

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn simd_scan_item_headers(payload: &[u8], start: usize) -> Vec<usize> {
    // SIMD 扫描策略: 
    // D2R item header: 低 4 bits of flags == 0b? (某个模式)
    // version 字段在固定位置
    // 
    // 简化策略: 用 SSE2 并行检查多个 byte 的 bits[4] 和 bits[0]
    // 组合成候选位置，再用完整 BitReader 验证
    
    let mut results = Vec::new();
    let chunks = payload.chunks(16);
    
    for (chunk_idx, chunk) in chunks.enumerate() {
        let mut bytes = [0u8; 16];
        bytes[..chunk.len()].copy_from_slice(chunk);
        
        // SSE2: 加载 16 bytes
        let data = unsafe { _mm_loadu_si128(bytes.as_ptr() as *const _) };
        
        // 检查每个 byte 的 bit[4] (flags bit) 和 bit[0] (version LSB)
        // 组合预测: flags_bit4 == 1 && version_bit0 == 0 可能是 item header
        // 
        // 实际需要根据 D2R 格式调整模式
        
        // ... 提取候选位置，加入 results
    }
    
    results
}
```

**更实用的方案 — 并行扫描 + 提前退出**:

```rust
/// 使用 Rayon 并行扫描多个候选位置
pub fn parallel_scan_next_item(
    payload: &[u8], 
    start: usize, 
    config: &ScanConfig,
) -> Option<ScanResult> {
    use rayon::prelude::*;
    
    let max_scan = std::cmp::min(config.max_scan_bytes, 
        (payload.len() * 8).saturating_sub(start + 35) / 8);
    
    // 生成候选位置列表
    let candidates: Vec<usize> = (1..=max_scan)
        .map(|skip| start + skip * 8)
        .filter(|&&p| p + 80 <= payload.len() * 8)
        .collect();
    
    // Rayon 并行验证
    candidates.par_iter()
        .find_any(|&&probe| is_valid_item_header(payload, probe, config))
        .map(|&probe| ScanResult { position: probe })
}

// 检查是否是有效 item header（无 BitReader 分配，直接 byte 操作）
#[inline]
fn is_valid_item_header(payload: &[u8], probe: usize, config: &ScanConfig) -> bool {
    let byte_pos = probe / 8;
    if byte_pos + 10 > payload.len() { return false; }
    
    // 直接从 bytes 读取 flags (32 bits from bits 0-31)
    // D2R LSB-first: byte[0] 的 bit[0] 是 flags bit[0]
    let w0 = payload[byte_pos] as u32
        | (payload[byte_pos+1] as u32) << 8
        | (payload[byte_pos+2] as u32) << 16
        | (payload[byte_pos+3] as u32) << 24;
    
    let flags = w0;
    
    // 检查 flags 约束
    if flags.count_ones() > config.max_flag_bits { return false; }
    if ((flags >> 4) & 1) != 1 { return false; }  // bit[4] must be 1
    
    // 直接读 version (bits 32-34)
    let ver = ((w0 >> 32) & 0x7) as u8;  // 如果有下一个 byte
    // 实际上 version 跨字节，需要更复杂的 byte-level 读取
    // 这里只是示意，实际需要按 LSB-first bit 顺序提取 bits[32..35]
    
    true
}
```

**更现实方案 — 批量预过滤**:

```rust
/// 先用简单 byte-level 预过滤快速排除不可能的位置
/// 再对候选位置用完整 BitReader 验证
pub fn scan_next_item_fast(
    payload: &[u8], 
    start: usize, 
    config: &ScanConfig,
) -> Option<ScanResult> {
    let max_scan = std::cmp::min(config.max_scan_bytes, 
        (payload.len() * 8).saturating_sub(start + 35) / 8);
    
    // Step 1: 快速预过滤 — 只检查 byte 级别的模式
    let candidates: Vec<usize> = (1..=max_scan)
        .map(|skip| start + skip * 8)
        .filter(|&probe| quick_header_check(payload, probe))
        .collect();
    
    // Step 2: 对候选位置用 BitReader 精确验证
    for probe in candidates {
        let mut pr = BitReader::new(payload);
        pr.seek(probe);
        // ... 完整验证
        if is_valid { return Some(ScanResult { position: probe }); }
    }
    
    None
}

/// 快速预过滤: 只检查 flags 的前几个 bits
#[inline]
fn quick_header_check(payload: &[u8], probe: usize) -> bool {
    let byte_pos = probe / 8;
    if byte_pos + 4 > payload.len() { return false; }
    
    // 只检查最低 byte 的 bit[4] (flags[4] == 1)
    // 和 flags.count_ones() <= 6
    let byte = payload[byte_pos];
    if (byte & 0x10) == 0 { return false; }  // bit[4] must be set
    
    // count_ones 很快，但不是零成本
    // 可以放宽条件，让后续验证做最终判断
    true
}
```

**预期收益**: 减少 50-90% 的无效 BitReader 创建。Page 1 的 scan_count=309，如果大部分被快速预过滤掉，scan_time 从 800ms 降到 100ms 以内。

### 2.3 Memory-Mapped File I/O

**原理**: 对大文件使用 `mmap`，避免一次性分配大 Vec 和多次复制。

```rust
use memmap2::Mmap;

pub fn parse_stash_mmap(path: &Path) -> Result<Vec<Page>, String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open: {}", e))?;
    
    // mmap: 不占用 heap 空间，按需 page fault 加载
    let mmap = unsafe { Mmap::map(&file) }
        .map_err(|e| format!("Failed to mmap: {}", e))?;
    
    // 直接用 &[u8] 作为 payload
    let data: &[u8] = &mmap;
    
    parse_pages(data)  // 现有解析逻辑无需大改
}
```

**收益**: 
- 对 10MB+ 的 stash 文件，减少内存复制
- 对稀疏访问场景（只读某些 page），按需加载
- 多个并发读取可以共享同一份物理内存页

**注意**: 当前测试的 fixture 只有 18KB，mmap 收益不明显。对用户的完整存档（可能有 50-200MB）效果显著。

**Rust crate**: `memmap2` (0.8.0+) 或标准库 `Vec` + `File::read` 对小文件足够快。

### 2.4 零拷贝 BitReader（中等优先级）

**问题**: 每次 `scan_next_item` 循环都 `BitReader::new(payload)` 分配新的 `BitVec`。

```rust
// 当前: 每次扫描都分配
let mut pr = BitReader::new(payload);  // 分配 BitVec
pr.seek(probe);
let pf = pr.read_u32(32);

// 优化: 直接在 byte 级别操作，不分配 BitVec
#[inline]
fn read_bits_from_bytes(bytes: &[u8], bit_offset: usize, n: usize) -> u32 {
    // 直接从 byte slice 读取，不创建 BitReader
    let byte_idx = bit_offset / 8;
    let bit_offs = bit_offset % 8;
    
    let mut word = u32::from_le_bytes([
        bytes.get(byte_idx).copied().unwrap_or(0),
        bytes.get(byte_idx + 1).copied().unwrap_or(0),
        bytes.get(byte_idx + 2).copied().unwrap_or(0),
        bytes.get(byte_idx + 3).copied().unwrap_or(0),
    ]);
    
    if bit_offs != 0 {
        word >>= bit_offs;
    }
    
    let mask = (1u32 << n) - 1;
    word & mask
}

// scan_next_item 改造后:
fn scan_next_item_zero_copy(payload: &[u8], start: usize, config: &ScanConfig) -> Option<ScanResult> {
    let max_scan = std::cmp::min(config.max_scan_bytes, 
        (payload.len() * 8).saturating_sub(start + 35) / 8);
    
    for skip in 1..=max_scan {
        let probe = start + skip * 8;
        if probe + 80 > payload.len() * 8 { break; }
        
        // 直接读 bits，不分配 BitReader
        let flags = read_bits_from_bytes(payload, probe, 32);
        let version = read_bits_from_bytes(payload, probe + 32, 3) as u8;
        
        let version_ok = if config.accept_version_or_less {
            version <= config.version
        } else {
            version == config.version
        };
        
        if !(flags.count_ones() <= config.max_flag_bits 
            && ((flags >> 4) & 1) == 1 
            && version_ok) 
        {
            continue;
        }
        
        // 只有通过预检查才创建 BitReader 读 code
        let code = decode_huffman_string(&mut BitReader::new(payload));
        // ... 后续验证
    }
    
    None
}
```

**预期收益**: 消除 scan 路径上 90% 的 BitReader 分配。Page 1 的 309 次 scan 变成零分配操作。

### 2.5 Rayon 并行页面解析

```rust
use rayon::prelude::*;

pub fn parse_all_pages_parallel(pages_info: &PagesInfo) -> Vec<(usize, Vec<ParsedItem>)> {
    pages_info.pages.par_iter()
        .filter(|p| jm_declared_count(&p.data).unwrap_or(0) > 0)
        .map(|page| {
            let declared = jm_declared_count(&page.data).unwrap_or(0);
            let items = parse_jm_page(&page.data, page.index, page.is_stackable);
            (page.index, declared, items)
        })
        .collect()
}
```

**注意**: 当前解析瓶颈是单线程的 scan_forward 开销，parallel 对 page 0/1/2 可能无效（它们是串行 forward-scan）。但对多个独立页面（如同时解析 page 0, 4, 6）可以并行。

**实际收益**: 多核机器上可能提速 2-4x（取决于页面数量和复杂度分布）。

### 2.6 预解析 Item Code 验证表

```rust
/// 替代 ALL_ITEMS 线性搜索 (jm_reader.rs:487-488)
/// 当前: O(n) 线性搜索整个 ALL_ITEMS 列表
/// 优化: O(1) HashSet 或固定数组索引

lazy_static::lazy_static! {
    static ref ITEM_CODE_SET: HashSet<&'static str> = {
        ALL_ITEMS.iter().map(|(c, _, _, _, _)| *c).collect()
    };
    static ref ITEM_CODE_TO_INFO: HashMap<&'static str, &'static (u8, u8, u8, u8)> = {
        ALL_ITEMS.iter().map(|(c, a, b, d, e)| (c, (a, b, d, e))).collect()
    };
}

// 使用:
let in_known = ITEM_CODE_SET.contains(pi.item.code.as_str());

// 如果还需要获取物品类型信息:
if let Some(info) = ITEM_CODE_TO_INFO.get(pi.item.code.as_str()) {
    // info.0 = category, info.1 = ... 
}
```

**收益**: `ALL_ITEMS` 有 600+ 条目，每次 parse 都线性搜索。HashSet 查找 O(1)，**提速 10-100x**。

### 2.7 统计表缓存与共享

```rust
// 当前: 每个 page 解析都重新 build StatTable
// 分析: parse_jm_page 每次调用都会在内部 build runtime table

// 优化: 在 jm_reader 模块级别缓存
lazy_static::lazy_static! {
    static ref STAT_TABLE_CACHE: StatTable = build_stat_table();
}

// parse_jm_page 改用缓存版本:
let table = if crate::data::stat_loader::has_runtime_table() {
    crate::data::stat_loader::build_runtime_table()  // 每次重新构建
} else {
    crate::data::stat_cost::build_stat_table()
};

// 改成:
let table = &*STAT_TABLE_CACHE;  // 零成本引用
```

当前测试输出显示 "build StatTable" 在每个 page 解析前都出现，说明 StatTable 没有被缓存。**每次 build StatTable 可能耗时 3-5ms**，对 10 个非空页面就是 30-50ms。

---

## 三、预期收益汇总

| 优化项 | 预估提速 | 难度 | 优先级 |
|--------|----------|------|--------|
| BitReader 批量操作 | 10-20x | 低 | **P0** |
| 零拷贝 scan | 5-10x | 低 | **P0** |
| 预过滤减少无效 scan | 3-8x | 低 | **P1** |
| HashSet 替代 ALL_ITEMS 线性搜索 | 10-100x | 低 | **P1** |
| StatTable 缓存 | 1.5-2x | 低 | **P1** |
| Rayon 并行页面 | 2-4x | 中 | P2 |
| mmap 大文件 | 1.2-2x | 低 | P2 |
| SIMD 扫描 | 2-5x | 高 | P3 |

**综合预估**: 实现 P0+P1 后，Page 1 的 scan_time 可能从 800ms 降到 50-100ms，整体解析速度提升 5-10x。

---

## 四、实施建议

### Phase 1: 消除不必要分配（低风险，高收益）

1. `BitReader` 添加 `read_u32_fast`, `read_u8_fast`, `read_u16_fast` 方法
2. `scan_next_item` 改用零拷贝 byte-level 操作，不创建 BitReader
3. 预过滤阶段只检查 byte[bit 4]，通过后再创建 BitReader
4. `ITEM_CODE_SET` HashSet 替代 ALL_ITEMS 线性搜索

### Phase 2: 批量操作 + 缓存

1. `StatTable` 模块级缓存
2. `scan_next_item` 批量预生成候选位置

### Phase 3: 并行化

1. 对多页面场景用 Rayon 并行解析
2. mmap 支持

### Phase 4: SIMD（可选）

只有在 Phase 1-2 后仍然不够快的情况下才考虑。
