#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use sha2::{Sha256, Digest};

risc0_zkvm::guest::entry!(main);

fn main() {
    // 1. 读取私有输入 (密码)
    // 我们读取为 String。注意：String 在 no_std 下需要 alloc 支持，
    // risc0_zkvm 默认提供了 allocator。
    let password: String = env::read();

    // 2. 计算 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();

    // result 是 GenericArray<u8, U32>，我们需要将其转换为 Vec<u8> 或数组以便 commit
    // 这里我们直接 commit 字节切片
    env::commit_slice(result.as_slice());
}
