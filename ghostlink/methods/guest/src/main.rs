#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};
use risc0_zkvm::sha::{Impl, Sha256};

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

risc0_zkvm::guest::entry!(main);

#[derive(Debug, Serialize, Deserialize)]
struct GithubUser {
    id: u64,
    login: String,
    created_at: String,
    public_repos: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GuestInput {
    user_json: String,
    recipient: [u8; 20],
}

fn main() {
    // 1. 读取输入
    let input: GuestInput = env::read();

    // 2. 解析 JSON
    let user: GithubUser = serde_json::from_str(&input.user_json).expect("Failed to parse JSON");

    // 3. 验证逻辑
    if user.public_repos == 0 {
        panic!("User has no public repos!");
    }

    let year_str = &user.created_at[0..4];
    let year: i32 = year_str.parse().expect("Invalid year format");

    if year >= 2023 {
        panic!("Account is too new! Must be created before 2023.");
    }

    // 4. 生成 Nullifier
    let nullifier_digest = *Impl::hash_bytes(&user.id.to_le_bytes());
    let nullifier: [u8; 32] = nullifier_digest.into();

    // 5. 构建 ABI 编码的 Journal
    // Solidity: abi.encode(address recipient, bytes32 nullifier)
    // ABI 编码格式：每个参数都是 32 字节，address 左填充 12 个零
    let mut journal = Vec::with_capacity(64);

    // Part A: Address (20 bytes) -> Padded to 32 bytes (left-padded with zeros)
    journal.extend_from_slice(&[0u8; 12]); // Padding
    journal.extend_from_slice(&input.recipient);

    // Part B: Nullifier (32 bytes)
    journal.extend_from_slice(&nullifier);

    // 6. 提交 Journal（合约会计算 sha256(journal) 作为 postStateDigest）
    env::commit_slice(&journal);
}
