// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]
#![no_std]

use risc0_zkvm::{
    guest::env,
    serde::to_vec,
    sha::{Impl, Sha256},
};
use voting_machine_core::{InitializeVotingMachineCommit, VotingMachineState};

// 定义 guest 程序的入口点
risc0_zkvm::guest::entry!(main);

/// 初始化投票机的 guest 程序
///
/// 这个程序在 ZKVM 中执行，用于：
/// 1. 读取初始投票机状态
/// 2. 计算状态的哈希摘要
/// 3. 提交初始化记录到 journal
///
/// 通过零知识证明，可以验证投票机确实是以指定的初始状态开始的，
/// 而不需要公开状态的完整信息。
fn main() {
    // 从 host 端读取投票机状态
    let state: VotingMachineState = env::read();
    //kyp 打印初始化状态
    tracing::info!("guest 初始化状态: {:?}", state);

    // 将状态序列化为字节数组
    let state_bytes = to_vec(&state).unwrap();
    // 计算状态的 SHA256 哈希摘要
    let state_hash = *Impl::hash_words(&state_bytes);

    // 提交初始化记录到 journal
    // 这个记录会被包含在证明收据中，任何人都可以验证
    env::commit(&InitializeVotingMachineCommit {
        polls_open: state.polls_open,
        voter_bitfield: state.voter_bitfield,
        state: state_hash,
    });
}
