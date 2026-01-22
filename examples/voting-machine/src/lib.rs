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

use risc0_zkvm::{default_prover, serde::from_slice, ExecutorEnv, Receipt, Result};
use voting_machine_core::{
    Ballot, FreezeVotingMachineCommit, FreezeVotingMachineParams, FreezeVotingMachineResult,
    InitializeVotingMachineCommit, SubmitBallotCommit, SubmitBallotParams, VotingMachineState,
};
use voting_machine_methods::{FREEZE_ELF, FREEZE_ID, INIT_ELF, INIT_ID, SUBMIT_ELF, SUBMIT_ID};

/// 初始化投票机的消息
/// 包含 RISC Zero 生成的证明收据，用于验证初始化操作的正确性
pub struct InitMessage {
    /// RISC Zero 生成的证明收据
    receipt: Receipt,
}

impl InitMessage {
    /// 从收据中解码获取初始化提交记录（不验证）
    pub fn get_state(&self) -> Result<InitializeVotingMachineCommit> {
        Ok(self.receipt.journal.decode()?)
    }

    /// 验证收据并获取初始化提交记录
    /// 验证收据的有效性，确保初始化操作是在 ZKVM 中正确执行的
    pub fn verify_and_get_commit(&self) -> Result<InitializeVotingMachineCommit> {
        self.receipt.verify(INIT_ID)?;
        self.get_state()
    }
}

/// 提交选票的消息
/// 包含 RISC Zero 生成的证明收据，用于验证选票提交操作的正确性
pub struct SubmitBallotMessage {
    /// RISC Zero 生成的证明收据
    receipt: Receipt,
}

impl SubmitBallotMessage {
    /// 从收据中解码获取选票提交记录（不验证）
    pub fn get_commit(&self) -> Result<SubmitBallotCommit> {
        Ok(self.receipt.journal.decode()?)
    }

    /// 验证收据并获取选票提交记录
    /// 验证收据的有效性，确保选票提交操作是在 ZKVM 中正确执行的
    pub fn verify_and_get_commit(&self) -> Result<SubmitBallotCommit> {
        //打印镜像id
        tracing::info!("镜像id: {:?}", SUBMIT_ID);
        self.receipt.verify(SUBMIT_ID)?;
        self.get_commit()
    }
}

/// 冻结投票机的消息
/// 包含 RISC Zero 生成的证明收据，用于验证冻结操作的正确性
pub struct FreezeStationMessage {
    /// RISC Zero 生成的证明收据
    receipt: Receipt,
}

impl FreezeStationMessage {
    /// 从收据中解码获取冻结提交记录（不验证）
    pub fn get_commit(&self) -> Result<FreezeVotingMachineCommit> {
        Ok(self.receipt.journal.decode()?)
    }

    /// 验证收据并获取冻结提交记录
    /// 验证收据的有效性，确保冻结操作是在 ZKVM 中正确执行的
    pub fn verify_and_get_commit(&self) -> Result<FreezeVotingMachineCommit> {
        self.receipt.verify(FREEZE_ID)?;
        self.get_commit()
    }
}

/// 投票站
/// 管理投票机的状态，并提供初始化、提交选票和冻结操作
#[derive(Debug)]
pub struct PollingStation {
    /// 当前的投票机状态
    state: VotingMachineState,
}

impl PollingStation {
    /// 创建新的投票站
    ///
    /// # 参数
    /// * `state` - 初始投票机状态
    pub fn new(state: VotingMachineState) -> Self {
        PollingStation { state }
    }

    /// 初始化投票机
    ///
    /// 在 ZKVM 中执行初始化操作，生成证明收据。
    /// 收据可以用于验证投票机的初始状态，确保投票站以正确的状态开始。
    ///
    /// # 返回
    /// * `InitMessage` - 包含初始化证明收据的消息
    pub fn init(&self) -> Result<InitMessage> {
        //kyp 打印初始化状态
        tracing::info!("abc初始化状态: {:?}", self.state);

        tracing::info!("init");
        // 构建执行环境，将状态写入 guest 端
        let env = ExecutorEnv::builder().write(&self.state)?.build()?;
        // 使用默认证明器生成证明
        let prover = default_prover();
        // 执行 guest 程序（init.rs）并生成证明收据
        let receipt = prover.prove(env, INIT_ELF)?.receipt;

        //kyp 打印初始化提交记录
        tracing::info!(
            "初始化 init commit: {:?}",
            receipt.journal.decode::<InitializeVotingMachineCommit>()?
        );

        Ok(InitMessage { receipt })
    }

    /// 提交选票
    ///
    /// 在 ZKVM 中执行选票提交操作，生成证明收据。
    /// 收据可以用于验证选票是否被正确处理和计入。
    ///
    /// # 参数
    /// * `ballot` - 要提交的选票
    ///
    /// # 返回
    /// * `SubmitBallotMessage` - 包含选票提交证明收据的消息
    pub fn submit(&mut self, ballot: &Ballot) -> Result<SubmitBallotMessage> {
        //kyp 打印提交选票
        tracing::info!("提交选票状态: {:?}", self);
        tracing::info!("提交选票 submit: {:?}", ballot);
        // 创建提交参数
        let params: SubmitBallotParams =
            SubmitBallotParams::new(self.state.clone(), ballot.clone());
        // 用于接收 guest 端的输出
        let mut output = Vec::new();
        // 构建执行环境，将参数写入 guest 端，并设置输出流
        let env = ExecutorEnv::builder()
            .write(&params)?
            .stdout(&mut output)
            .build()?;
        // 使用默认证明器生成证明
        let prover = default_prover();
        // 执行 guest 程序（submit.rs）并生成证明收据
        let receipt = prover.prove(env, SUBMIT_ELF)?.receipt;
        //kyp 打印提交选票收据
        tracing::info!("提交选票收据: {:?}", receipt);
        // 从输出中解码更新后的状态
        self.state = from_slice(&output)?;
        //kyp 打印提交选票结果
        tracing::info!("提交选票结果: {:?}", self.state);
        Ok(SubmitBallotMessage { receipt })
    }

    /// 冻结投票机
    ///
    /// 在 ZKVM 中执行冻结操作，关闭投票站并停止接受新投票。
    /// 收据可以用于验证投票机已正确冻结，并显示最终票数。
    ///
    /// # 返回
    /// * `FreezeStationMessage` - 包含冻结证明收据的消息
    pub fn freeze(&mut self) -> Result<FreezeStationMessage> {
        tracing::info!("freeze");
        // 创建冻结参数
        let params = FreezeVotingMachineParams::new(self.state.clone());
        // 用于接收 guest 端的输出
        let mut output = Vec::new();
        // 构建执行环境，将参数写入 guest 端，并设置输出流
        let env = ExecutorEnv::builder()
            .write(&params)?
            .stdout(&mut output)
            .build()?;
        // 使用默认证明器生成证明
        let prover = default_prover();
        // 执行 guest 程序（freeze.rs）并生成证明收据
        let receipt = prover.prove(env, FREEZE_ELF)?.receipt;
        // 从输出中解码更新后的状态
        let result: FreezeVotingMachineResult = from_slice(&output)?;
        //kyp 打印冻结结果
        tracing::info!("冻结结果: {:?}", result);
        self.state = result.state;
        Ok(FreezeStationMessage { receipt })
    }
}

//--------------------------------------

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;

    /// 测试投票协议流程
    ///
    /// 测试场景：
    /// 1. 初始化投票机
    /// 2. 投票者0投"否"票
    /// 3. 投票者1投"是"票
    /// 4. 投票者2投"是"票
    /// 5. 投票者1尝试再次投票（应该失败，因为已投过票）
    /// 6. 投票者3投"否"票
    /// 7. 冻结投票机
    /// 8. 投票者4尝试投票（应该失败，因为投票站已关闭）
    ///
    /// 预期结果：最终"是"票数为2（投票者1和2）
    #[test]
    fn protocol() {
        // 创建初始状态：投票站开放，无投票者，票数为0
        let polling_station_state: VotingMachineState = VotingMachineState {
            polls_open: true,
            voter_bitfield: 0,
            count: 0,
        };

        let mut polling_station: PollingStation = PollingStation::new(polling_station_state);

        // 创建测试选票
        let ballot1 = Ballot {
            voter: 0,
            vote_yes: false,
        };
        let ballot2 = Ballot {
            voter: 1,
            vote_yes: true,
        };
        let ballot3 = Ballot {
            voter: 2,
            vote_yes: true,
        };
        // 投票者1尝试重复投票
        let ballot4 = Ballot {
            voter: 1,
            vote_yes: false,
        };
        let ballot5 = Ballot {
            voter: 3,
            vote_yes: false,
        };
        // 在投票站关闭后尝试投票
        let ballot6 = Ballot {
            voter: 4,
            vote_yes: true,
        };

        // 执行投票流程
        let init_msg: InitMessage = polling_station.init().unwrap();
        let ballot_msg1: SubmitBallotMessage = polling_station.submit(&ballot1).unwrap();
        let ballot_msg2: SubmitBallotMessage = polling_station.submit(&ballot2).unwrap();
        let ballot_msg3: SubmitBallotMessage = polling_station.submit(&ballot3).unwrap();
        let ballot_msg4: SubmitBallotMessage = polling_station.submit(&ballot4).unwrap();
        let ballot_msg5: SubmitBallotMessage = polling_station.submit(&ballot5).unwrap();
        let close_msg: FreezeStationMessage = polling_station.freeze().unwrap();
        let ballot_msg6: SubmitBallotMessage = polling_station.submit(&ballot6).unwrap();

        // 验证最终票数：应该是2（投票者1和2投了"是"票）
        assert_eq!(polling_station.state.count, 2);

        // 验证所有收据的有效性
        let init_state = init_msg.verify_and_get_commit();
        let ballot_commit1 = ballot_msg1.verify_and_get_commit();
        let ballot_commit2 = ballot_msg2.verify_and_get_commit();
        let ballot_commit3 = ballot_msg3.verify_and_get_commit();
        let ballot_commit4 = ballot_msg4.verify_and_get_commit();
        let ballot_commit5 = ballot_msg5.verify_and_get_commit();
        let close_state = close_msg.verify_and_get_commit();
        let ballot_commit6 = ballot_msg6.verify_and_get_commit();

        tracing::info!("initial commit: {:?}", init_state);
        tracing::info!("ballot 1: {:?}", ballot1);
        tracing::info!("ballot 1 commit: {:?}", ballot_commit1);
        tracing::info!("ballot 2: {:?}", ballot2);
        tracing::info!("ballot 2 commit: {:?}", ballot_commit2);
        tracing::info!("ballot 3: {:?}", ballot3);
        tracing::info!("ballot 3 commit: {:?}", ballot_commit3);
        tracing::info!("ballot 4: {:?}", ballot4);
        tracing::info!("ballot 4 commit: {:?}", ballot_commit4);
        tracing::info!("ballot 5: {:?}", ballot5);
        tracing::info!("ballot 5 commit: {:?}", ballot_commit5);
        tracing::info!("freeze commit: {:?}", close_state);
        tracing::info!("ballot 6: {:?}", ballot6);
        tracing::info!("ballot 6 commit: {:?}", ballot_commit6);
        tracing::info!("Final vote count: {:?}", polling_station.state.count);
    }
}
