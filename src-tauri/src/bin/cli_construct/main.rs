//! D2S 角色存档 CLI 查看器 — Python cli_construct.py 的 Rust 移植。
//!
//! Usage:
//!     cargo run --bin cli_construct -- <path-to.d2s> [--json] [--detail] [--bits] [--watch]
//!
//! 功能：
//! - 显示角色摘要（名称/职业/等级/属性/技能/任务/小站/物品）
//! - --json：输出 JSON
//! - --detail：显示完整 stat 列表
//! - --bits [N]：显示物品位级布局（无 N = 物品列表，-b N = 第 N 个物品）
//! - --watch / -w：监控文件变化自动重新解析

use std::path::Path;

mod construct;
mod display;
mod display_names;
mod dump_item;
mod helpers;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut path_str = String::new();
    let mut json_mode = false;
    let mut detail = false;
    let mut watch = false;
    let mut interval = 2.0f64;
    let mut bits: Option<i32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--detail" | "-d" => detail = true,
            "--watch" | "-w" => watch = true,
            "--interval" => {
                i += 1;
                if i < args.len() { interval = args[i].parse().unwrap_or(2.0); }
            }
            "--bits" | "-b" => {
                if i + 1 < args.len() && !args[i+1].starts_with('-') {
                    i += 1;
                    bits = Some(args[i].parse::<i32>().unwrap_or(-1));
                } else {
                    bits = Some(-1);
                }
            }
            _ => {
                if path_str.is_empty() {
                    path_str = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if path_str.is_empty() {
        eprintln!("用法: cli_construct <path-to.d2s> [--json] [--detail] [--bits [N]] [--watch]");
        std::process::exit(1);
    }

    let path = Path::new(&path_str);
    if !path.exists() {
        eprintln!("错误: 文件不存在: {}", path_str);
        std::process::exit(1);
    }

    if watch {
        construct::watch_loop(path, json_mode, detail, interval);
    } else {
        construct::run_once(path, json_mode, detail, bits);
    }
}
