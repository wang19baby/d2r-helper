# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.0] — 2026-08-01

### Features

- **Tooltip system** — Full magical property decoding with Chinese translations (itemstatcost 368/368 covered)
- **Equipment tooltips** — Set bonuses, unique/set level requirements, skill tab names, armor defense, attack speed
- **Character page** — D2S parser with skills/progression summary, 8 class portraits
- **Unified Vault (收藏库)** — Dual-grid drag-and-drop, item notes/tags, per-code default page
- **Storage Workbench** — Stackable items deposit/withdraw with quantity selection, mod stash support
- **Marketplace** — Token economy, listings, auto-sell timer, cancel/buy flow
- **Protocol layer** — BitReader/Writer, Huffman encoding, D2I/D2S bit-level parsing
- **Name resolver** — Profile-based caching (600-1100ms warmup reduction)
- **Auto-backup** — Stash files backed up before any write

### Bug Fixes

- Page 0 edits preserved on stackable deposit
- Merc segment raw extraction with kf boundary
- Socketed items preserved end-to-end (item_json, withdraw, page=5)
- Stat 151 misread as fire resist
- Dex requirements in tooltips
- Attack speed color (affix blue vs base white)
- Stat formatting DB priority

### Refactor

- D2S items segment unified under jm_reader
- Character segment merged (items/items_new)
- NameResolver cache per profile
