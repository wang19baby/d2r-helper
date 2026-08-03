# 仓库存入/取回端到端测试设计

**日期:** 2026-07-27
**作者:** Claude (brainstorming 产出)
**状态:** 设计待审,未实施

## 背景

`warehouse_deposit` 和 `warehouse_withdraw` 是直接修改 d2i 存档的两条命令,涉及:

- 修改游戏存档 (`.d2i` 共享仓库)
- 修改 SQLite 仓库
- 创建/读取 auto-backup
- 解析/重建 raw item bits

这两条命令在 `src-tauri/src/commands/warehouse.rs:212` 和 `:496`,是当前最大的数据安全风险。
当前 `src-tauri/tests/warehouse_tests.rs` **完全没**测过它们,只覆盖了 database CRUD。

## 范围

按 2026-07-27 brainstorming 会议确认:

- **覆盖范围:** 端到端 (含 happy path + round-trip + 边界)
- **测试数据:** 真实 fixture 副本 + 动态构造最小 d2i 都用
- **精度:** 测 raw bits round-trip (deposit → withdraw → deposit 字节级一致)

## 测试金字塔

```text
        边界层 (8 用例, T11-T18)
       ─────────────────────────
      round-trip 层 (4 用例, T07-T10)
     ─────────────────────────────
    happy path 层 (6 用例, T01-T06)
   ─────────────────────────────────
  基础设施 (2 helper, fixture 复用)
```

## 基础设施

### Helper 1: `copy_fixture_to_temp(name: &str) -> PathBuf`

```rust
fn copy_fixture_to_temp(name: &str) -> PathBuf {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ModernSharedStashSoftCoreV2.d2i");
    let dst = std::env::temp_dir().join(format!("d2r_e2e_{}_{}.d2i", std::process::id(), name));
    std::fs::copy(&src, &dst).expect("Failed to copy fixture");
    dst
}
```

- 真实 fixture:`tests/fixtures/ModernSharedStashSoftCoreV2.d2i` (1.5 MB,page[0] 80 装备,page[1] 堆叠)
- 每次测试独立文件,避免并发干扰
- 测试结束 `cleanup_test_d2i(path)` 删除

### Helper 2: `build_minimal_stackable_d2i(items: Vec<(code, qty)>) -> Vec<u8>`

动态构造最小可解析 d2i,只含 1 个 stackable 页 + JM items。

- 用于边界:空 stash、单 item、max qty
- **风险:** header/tail 字节若错 1 位,`split_legacy_d2i_pages` 会 fail
- **验证:** helper 写完必须立即跑 1 个 sanity test,确认 parse 成功
- **回退:** 如果动态构造太难,边界测试可以改用真实 fixture 但通过 `find_item_by_code` 定位

### Helper 3: `parse_stash_items(path) -> Vec<(usize, StashItem)>`

```rust
fn parse_stash_items(path: &Path) -> Vec<(usize, StashItem)> {
    let data = std::fs::read(path).expect("read d2i");
    let pages = split_legacy_d2i_pages(&data).expect("parse d2i");
    read_all_stash_items(&pages.pages).expect("read items")
}
```

### Helper 4: `find_item_by_code<'a>(items, code) -> Option<&'a StashItem>`

### Helper 5: `open_test_db(name) -> Database`

直接复用现有 `warehouse_tests.rs:16` 的 `create_test_db`。

### Helper 6: `build_app_state(db: Database) -> AppState`

`warehouse_deposit` 接受 `State<AppState>`,需要构造:

```rust
fn build_app_state(db: Database) -> AppState {
    AppState {
        db: std::sync::Mutex::new(db),
        import_state: std::sync::Arc::new(std::sync::Mutex::new(ImportState::new())),
    }
}
```

要求 `AppState` 字段 public。当前 `src-tauri/src/lib.rs:82` 已经是 `pub db` / `pub import_state`,可以直接构造。

## 测试用例

### Happy path (6 用例)

| ID | 名称 | 输入 | 断言 |
|---|---|---|---|
| T01 | `deposit_full_removal_stackable` | 1 个 r01 x10,qty=10 | d2i stackable 页 items 数从 1→0;db 1 条 record,quantity=10,raw_item_bits 非空 |
| T02 | `deposit_partial_qty_stackable` | 1 个 r01 x20,qty=5 | d2i 仍 1 个 r01,amount=15;db quantity=5;**验证 px/py 二进制字段正确** |
| T03 | `withdraw_to_empty_cell` | 取 1 个 r01 → page=0, (0,0) | d2i items +1;db record 删除;auto-backup 文件存在 |
| T04 | `withdraw_preserves_other_items` | stash 2 items,取走 1 | 剩余 item 字节与原始一致 (除 amount/position);jm_reader 重新解析成功 |
| T05 | `round_trip_raw_bits` | deposit 1 个 rune → withdraw → deposit 再次 | 两次 raw_item_bits 字节级一致 (除 amount 字段) |
| T06 | `round_trip_with_different_position` | deposit → withdraw 到 (5,5) | 新 stash 中 item 位于 (5,5);`update_item_position` 正确 |

### Round-trip (4 用例,真实 fixture)

| ID | 名称 | 步骤 | 断言 |
|---|---|---|---|
| T07 | `fixture_deposit_reduces_item_count` | 记录前 items 总数 N,deposit 已知存在的 code | deposit 后总数 = N (部分) 或 N-1 (全扣);db 记录 raw_item_bits > 0 |
| T08 | `fixture_withdraw_increases_count` | 取回 1 个 → item_count 最少页最右角 | items +1;stash 大小 > 原始;auto-backup == 原始字节 |
| T09 | `fixture_round_trip_idempotent` | deposit + withdraw (相同 code) | stash items 总数回到 deposit 之前;仓库空 |
| T10 | `fixture_d2i_byte_level_backup_match` | sha256(deposit 前 d2i) → 触发 deposit | auto-backup 文件内容 == sha256 期望值 (字节级) |

### 边界 (8 用例)

| ID | 名称 | 输入 | 期望 |
|---|---|---|---|
| T11 | `deposit_nonexistent_item_code` | code="xyz" | Err "Item 'xyz' not found";stash sha256 不变;db 无新增 |
| T12 | `deposit_zero_quantity` | qty=0 | Err "Quantity must be > 0";stash + db 都不变 |
| T13 | `deposit_invalid_page_index` | page=999 | Err "Page 999 not found" |
| T14 | `deposit_qty_exceeds_available` | stash r01 x5,deposit qty=10 | actual_qty=min(10,5)=5;stash r01 amount=0;db quantity=5。**注释记录此行为为契约** |
| T15 | `withdraw_item_id_not_in_db` | id="nonexistent" | Err "Warehouse item not found";stash 不变 |
| T16 | `withdraw_out_of_bounds_position` | x=100, y=100 | Err 含 "out of bounds";如果当前不验证,这个 case 变红 → 暴露 P0 bug |
| T17 | `withdraw_position_collision` | stash (3,3) 有 item,withdraw 2x2 item 到 (3,3) | 当前实现直接覆盖 → 测试断言当前 buggy 行为 + TODO 注释。**这是 review 提到的 P0** |
| T18 | `withdraw_to_equipment_page` | withdraw stackable 到 non-stackable 页 | 不 panic,返回 Err。当前行为未知,测试用来记录 |

## 失败恢复 (隐含)

- auto-backup 已在 T03/T08/T10 中验证
- 不单独测"stash 写失败时 db 回滚",因为**当前没这个机制**,这是下个 sprint 的 P0

## T17 (collision) 和 T18 (equipment page) 的 P0 含义

T17/T18 预期会 FAIL,这本身是有价值的:

- 把"已知风险"从"代码注释 + review 报告"升级为"测试套件红"
- 下个 sprint 修复 occupancy check 时,这些 case 转绿 = 修复完成的明确信号
- CI 跑 1 遍就能定位所有 P0 风险点

## 不在本工单

- 修复 T17/T18 揭示的 P0 bug
- 添加 helper `build_minimal_equipment_d2i` (动态构造 non-stackable 太复杂,先跳过)
- 性能测试 (deposit/withdraw 1000+ items 批量) — 不在 P0 数据安全范围

## 验收

```bash
# 跑新测试
cd src-tauri && cargo test --test warehouse_deposit_withdraw_e2e
# 期望:18 个用例
#  - T01-T06, T11-T15 全部 PASS
#  - T07-T10 (fixture) PASS (依赖 fixture 真实可解析)
#  - T16, T17, T18 状态待定,可能 FAIL 暴露 P0

# 全量回归
cd src-tauri && cargo test
cd web && npm test
# 期望:无回归
```

## 工作量估算

- 写 helper (含动态构造 sanity test): 0.5 天
- T01-T06 happy path: 0.5 天
- T07-T10 round-trip: 0.25 天
- T11-T18 边界: 0.25 天
- 总计: 约 1.5 天

## 风险

- 动态构造 d2i 的 header 字节精度:可能需要 1-2 轮迭代
- AppState 构造需要 field public:当前已经 public,无风险
- 真实 fixture 变更:ModernSharedStashSoftCoreV2.d2i 是 vendored 副本,变更需要同步更新
