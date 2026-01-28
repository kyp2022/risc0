#!/bin/bash

# 进入项目目录
cd /Users/ppg/Desktop/zkvm/risc0/ghostlink/host

# 设置环境变量，强制使用 rustup 的 Rust
export PATH="$HOME/.rustup/toolchains/1.89-aarch64-apple-darwin/bin:$HOME/.cargo/bin:$PATH"

# 显示当前使用的 rustc
echo "Using rustc at: $(which rustc)"
echo "Rustc version: $(rustc --version)"

# 运行 wasm-pack
wasm-pack build --target web --out-dir ../web/pkg
