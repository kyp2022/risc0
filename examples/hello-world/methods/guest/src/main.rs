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

// 禁用标准main函数，使用自定义入口点
#![no_main]
// 禁用标准库，使用no_std环境（适用于嵌入式或受限环境）
#![no_std]

// 导入zkVM guest环境，用于与host通信
use risc0_zkvm::guest::env;

// 定义zkVM guest程序的入口点为main函数
risc0_zkvm::guest::entry!(main);

fn main() {
    // 从host环境中读取第一个数字
    let a: u64 = env::read();
    // 从host环境中读取第二个数字
    let b: u64 = env::read();

    // 验证两个数都不为1（即非平凡因子）
    // 如果任一数为1，则panic并输出错误信息
    if a == 1 || b == 1 {
        panic!("Trivial factors")
    }

    // 计算乘积，同时小心处理整数溢出
    // checked_mul方法会在溢出时返回None，触发expect中的panic
    let product = a.checked_mul(b).expect("Integer overflow");

    // 将结果提交到zkVM的journal中
    // journal是zkVM证明的一部分，允许外部验证者读取结果
    // 而无需了解输入值a和b
    env::commit(&product);
}
