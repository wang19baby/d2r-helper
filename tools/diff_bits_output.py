#!/usr/bin/env python3
"""Compare Python cli_construct --bits -1 output with Rust debug_item_table output."""

import subprocess, sys, re
from pathlib import Path

D2S_PATH = Path(r"D:\work_space\personal_workspace\d2r\开心邪帝.d2s")
D2R_ZERO = Path(r"D:\work_space\personal_workspace\d2r-zero")
TAURI_DIR = Path(r"D:\work_space\personal_workspace\d2r\d2r-marketplace-tauri\src-tauri")

def get_python_output():
    result = subprocess.run(
        [sys.executable, "-m", "src.d2r_zero.cli_construct", "--bits", "-1", str(D2S_PATH)],
        capture_output=True, text=True, cwd=str(D2R_ZERO), timeout=30
    )
    return result.stdout

def get_rust_output():
    # Use cargo run to print debug_item_table output
    # For now, let's create a test binary approach
    result = subprocess.run(
        ["cargo", "run", "--bin", "debug_dump", "--", str(D2S_PATH)],
        capture_output=True, text=True, cwd=str(TAURI_DIR), timeout=120
    )
    return result.stdout

if __name__ == "__main__":
    # Just show Python output for now
    py_out = get_python_output()
    print("=== Python output ===")
    print(py_out[:2000])
    print("... (truncated)")
