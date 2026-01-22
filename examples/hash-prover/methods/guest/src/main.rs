#![no_main]
#![no_std]

extern crate alloc;
use alloc::string::String;
use risc0_zkvm::guest::env;
use sha2::{Sha256, Digest};

risc0_zkvm::guest::entry!(main);

fn main() {
    // 1. 读取私有输入 (密码)
    let password: String = env::read();

    // 2. 计算 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();

    // 3. Commit 原始哈希值 (32字节)
    env::commit_slice(result.as_slice());
}
