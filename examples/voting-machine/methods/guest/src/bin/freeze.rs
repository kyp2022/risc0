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
use voting_machine_core::{FreezeVotingMachineCommit, FreezeVotingMachineParams};

// 定义 guest 程序的入口点
risc0_zkvm::guest::entry!(main);

/// 冻结投票机的 guest 程序
/// 
/// 这个程序在 ZKVM 中执行，用于：
/// 1. 读取当前投票机状态
/// 2. 将投票站设置为关闭状态
/// 3. 计算冻结前后的状态哈希摘要
/// 4. 将更新后的状态写回 host 端
/// 5. 提交冻结记录到 journal，包含最终票数
/// 
/// 通过零知识证明，可以验证：
/// - 投票机确实被冻结（polls_open 变为 false）
/// - 状态转换的正确性（从开放状态到冻结状态）
/// - 最终票数的正确性（count 字段）
fn main() {
    // 从 host 端读取冻结参数（包含当前状态）
    let params: FreezeVotingMachineParams = env::read();
    
    // 在 ZKVM 中处理冻结逻辑（将 polls_open 设置为 false）
    let result = params.process();
    
    // 将更新后的状态写回 host 端（通过 stdout）
    env::write(&result.state);
    
    // 计算冻结前状态的哈希摘要
    let old_state_bytes = to_vec(&params.state).unwrap();
    let old_state_hash = *Impl::hash_words(&old_state_bytes);
    
    // 计算冻结后状态的哈希摘要
    let new_state_bytes = to_vec(&result.state).unwrap();
    let new_state_hash = *Impl::hash_words(&new_state_bytes);
    
    // 提交冻结记录到 journal
    // 这个记录会被包含在证明收据中，任何人都可以验证冻结操作和最终票数
    env::commit(&FreezeVotingMachineCommit {
        old_state: old_state_hash,
        new_state: new_state_hash,
        polls_open: result.state.polls_open,
        voter_bitfield: result.state.voter_bitfield,
        count: result.state.count,
    });
}
