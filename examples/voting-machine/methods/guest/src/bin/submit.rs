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
use voting_machine_core::{SubmitBallotCommit, SubmitBallotParams};

// 定义 guest 程序的入口点
risc0_zkvm::guest::entry!(main);

/// 提交选票的 guest 程序
///
/// 这个程序在 ZKVM 中执行，用于：
/// 1. 读取当前状态和选票
/// 2. 在受保护的环境中处理投票逻辑（防止重复投票、检查投票站状态等）
/// 3. 计算状态转换前后的哈希摘要
/// 4. 将更新后的状态写回 host 端
/// 5. 提交投票记录到 journal
///
/// 通过零知识证明，可以验证：
/// - 投票逻辑被正确执行（没有重复投票、投票站状态检查等）
/// - 状态转换的正确性（从旧状态到新状态）
/// - 投票是否被计入
fn main() {
    // 从 host 端读取提交参数（包含当前状态和选票）
    let params: SubmitBallotParams = env::read();

    //kyp 打印提交选票参数
    tracing::info!("guest 提交选票参数: {:?}", params);

    // 在 ZKVM 中处理投票逻辑
    // 这会检查投票站是否开放、投票者是否已投票等
    let result = params.process();
    //kyp 打印提交选票结果
    tracing::info!("guest 提交选票结果: {:?}", result);

    // 将更新后的状态写回 host 端（通过 stdout）
    env::write(&result.state);

    // 计算旧状态的哈希摘要
    let old_state_bytes = to_vec(&params.state).unwrap();
    let old_state_hash = *Impl::hash_words(&old_state_bytes);

    // 计算新状态的哈希摘要
    let new_state_bytes = to_vec(&result.state).unwrap();
    let new_state_hash = *Impl::hash_words(&new_state_bytes);

    // 提交投票记录到 journal
    // 这个记录会被包含在证明收据中，任何人都可以验证投票的正确性
    env::commit(&SubmitBallotCommit {
        old_state: old_state_hash,
        new_state: new_state_hash,
        polls_open: result.state.polls_open,
        voter_bitfield: result.state.voter_bitfield,
        voter: params.ballot.voter,
        vote_yes: params.ballot.vote_yes,
        vote_counted: result.vote_counted,
    });
}
