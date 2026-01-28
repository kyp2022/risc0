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

#![cfg_attr(not(test), no_std)]

use risc0_zkp::core::digest::Digest;
use serde::{Deserialize, Serialize};

/// 投票机状态结构体
/// 用于跟踪投票机的当前状态
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VotingMachineState {
    /// 投票是否开放：true 表示投票站开放，false 表示已冻结
    pub polls_open: bool,
    /// 投票者位域：使用位掩码记录哪些投票者已经投过票
    /// 例如：0b111 表示前3位投票者（0, 1, 2）已经投票
    pub voter_bitfield: u32,
    /// 投"是"的票数统计
    pub count: u32,
}

impl VotingMachineState {
    /// 处理投票操作
    ///
    /// # 参数
    /// * `voter` - 投票者ID（0-31，因为使用u32位域）
    /// * `vote_yes` - 是否投"是"票
    ///
    /// # 返回
    /// * `true` - 投票成功并被计入
    /// * `false` - 投票失败（投票站已关闭或该投票者已投过票）
    pub fn vote(&mut self, voter: u32, vote_yes: bool) -> bool {
        let mut vote_counted = false;
        // 创建投票者对应的位掩码，例如 voter=2 时，mask = 0b100 (1 << 2)
        let voter_mask = 1 << voter;
        // 检查投票站是否开放，且该投票者是否还未投票
        if self.polls_open && 0 == self.voter_bitfield & voter_mask {
            // 标记该投票者已投票
            self.voter_bitfield |= voter_mask;
            // 如果是"是"票，增加计数
            if vote_yes {
                self.count += 1;
            }
            vote_counted = true
        }
        vote_counted
    }
}

/// 初始化投票机的提交记录
/// 用于记录投票机的初始状态，包含状态的哈希摘要
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InitializeVotingMachineCommit {
    /// 投票站是否开放
    pub polls_open: bool,
    /// 投票者位域
    pub voter_bitfield: u32,
    /// 状态的哈希摘要，用于验证状态完整性
    pub state: Digest,
}

/// 选票结构体
/// 表示一张选票的信息
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Ballot {
    /// 投票者ID
    pub voter: u32,
    /// 是否投"是"票
    pub vote_yes: bool,
}

/// 提交选票的提交记录
/// 用于记录选票提交前后的状态变化，包含状态转换的证明
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitBallotCommit {
    /// 提交前的状态哈希
    pub old_state: Digest,
    /// 提交后的状态哈希
    pub new_state: Digest,
    /// 投票站是否开放
    pub polls_open: bool,
    /// 投票者位域
    pub voter_bitfield: u32,
    /// 投票者ID
    pub voter: u32,
    /// 是否投"是"票
    pub vote_yes: bool,
    /// 投票是否被计入
    pub vote_counted: bool,
}

/// 提交选票的参数
/// 包含当前状态和要提交的选票
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitBallotParams {
    /// 当前的投票机状态
    pub state: VotingMachineState,
    /// 要提交的选票
    pub ballot: Ballot,
}

/// 提交选票的结果
/// 包含处理后的状态和处理结果
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitBallotResult {
    /// 处理后的投票机状态
    pub state: VotingMachineState,
    /// 投票是否被计入
    pub vote_counted: bool,
    /// 是否投"是"票
    pub vote_yes: bool,
}

impl SubmitBallotParams {
    /// 创建提交选票参数
    pub fn new(state: VotingMachineState, ballot: Ballot) -> Self {
        //kyp 打印提交选票参数
        SubmitBallotParams { state, ballot }
    }

    /// 处理选票提交
    /// 在 ZKVM 的 guest 端执行，确保投票逻辑的正确性
    pub fn process(&self) -> SubmitBallotResult {
        let mut state = self.state.clone();
        //kyp 打印提交选票参数
        tracing::info!("core 提交选票参数2: {:?}", self);
        // 执行投票逻辑
        let vote_counted = state.vote(self.ballot.voter, self.ballot.vote_yes);
        //kyp 打印提交选票结果
        tracing::info!("core 提交选票结果: {:?}", vote_counted);
        SubmitBallotResult {
            state,
            vote_counted,
            vote_yes: self.ballot.vote_yes,
        }
    }
}

/// 冻结投票机的提交记录
/// 用于记录投票机冻结前后的状态变化和最终票数
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FreezeVotingMachineCommit {
    /// 冻结前的状态哈希
    pub old_state: Digest,
    /// 冻结后的状态哈希
    pub new_state: Digest,
    /// 投票站是否开放（冻结后应为 false）
    pub polls_open: bool,
    /// 投票者位域
    pub voter_bitfield: u32,
    /// 最终的"是"票数
    pub count: u32,
}

/// 冻结投票机的参数
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FreezeVotingMachineParams {
    /// 当前的投票机状态
    pub state: VotingMachineState,
}

/// 冻结投票机的结果
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FreezeVotingMachineResult {
    /// 冻结后的投票机状态
    pub state: VotingMachineState,
}

impl FreezeVotingMachineParams {
    /// 创建冻结投票机参数
    pub fn new(state: VotingMachineState) -> Self {
        FreezeVotingMachineParams { state }
    }

    /// 处理投票机冻结
    /// 将投票站设置为关闭状态，之后不再接受新的投票
    pub fn process(&self) -> FreezeVotingMachineResult {
        let mut state = self.state.clone();
        // 关闭投票站
        state.polls_open = false;
        FreezeVotingMachineResult { state }
    }
}
