# Contributing to D2R Helper

## Development Setup

```bash
# 1. Clone（本项目，D2R 助手）
git clone https://github.com/wang19baby/d2r-helper.git
cd d2r-helper

# 2. Install Rust (stable)
curl --proto ='https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install Node.js 20+
# https://nodejs.org/

# 4. Frontend dev
cd web && npm install && npm run dev

# 5. Rust check (from project root)
cd src-tauri && cargo check
```

## Required Checks Before Commit

**All must pass (or PR is rejected，CI 会跑 rust check + clippy + test + web build):**

```bash
# Rust
cd src-tauri && cargo check
cargo clippy --lib --bins -- -D warnings
cargo test --lib

# Frontend
cd web && npm ci && npm run build   # tsc -b + vite build
npm test
```

> 依赖真实游戏存档的测试（`tests/fixtures/` 中不随仓库分发的个人存档）在缺失时自动 SKIP。
> 协议解析细节与未对齐项见 `docs/protocol-d2s.md` / `docs/protocol-d2i.md`。

## Architecture Notes

- **Protocol layer** (`src-tauri/src/protocol/`) — bit-level D2I/D2S parsing; changes here affect save file integrity. Heavy comment coverage, do not optimize without benchmarks.
- **Warehouse deposit** (`commands/warehouse.rs`) — writes to both `.d2i` stash files and SQLite; always run the `warehouse_tests` suite after changes.
- **No external network calls** — this is a fully offline tool. Do not add network I/O without discussion.

## Commit Style

```
<type>(<scope>): <subject>

types: feat | fix | refactor | test | docs | chore
scope: tooltip | warehouse | protocol | d2i | d2s | stash | ...
```

Example: `feat(warehouse): add per-code default page fallback`

## Reporting Issues

- Include: OS version, D2R patch version, stash file version (v96/v97/v98/v105/v111)
- Attach a minimal `.d2i` that reproduces the issue (if not sensitive)
- Run `cargo test --lib` output in the issue description
