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

use composition_example_methods::{EXPONENTIATE_ELF, EXPONENTIATE_ID};
use hello_world::multiply;
use risc0_zkvm::{default_prover, ExecutorEnv};

fn main() {
    // 第一步：Alice 选择两个数字并相乘，生成一个 receipt（收据），证明 Alice 知道乘积的因式分解
    // 这类似于 RSA 密钥生成过程
    // multiply 函数会返回一个 receipt 和乘积 n
    let (multiply_receipt, n) = multiply(17, 23);

    // 第二步：Alice 将乘积 n 和证明她知道因式分解的 receipt 发送给 Bob
    // Bob 然后对一个秘密数字进行模幂运算：x^e mod n，其中 e 是公开指数，n 是 Alice 选择的合数
    // 这类似于 Bob 向 Alice 进行的 RSA 加密，整个过程由 zkVM 验证
    let env = ExecutorEnv::builder()
        // add_assumption 将需要验证的 receipt 提供给证明者
        // 这使得 guest 程序可以通过 env::verify 来验证这个 receipt
        .add_assumption(multiply_receipt)
        // 写入输入数据：(n, e, x) = (391, 9, 100)
        // n: 模数（Alice 选择的合数）
        // e: 公开指数
        // x: 秘密值（将被加密）
        .write(&(n, 9u64, 100u64))
        .unwrap()
        .build()
        .unwrap();

    // 第三步：生成证明，执行 guest 程序（EXPONENTIATE_ELF）
    // guest 程序会：
    // 1. 验证 multiply_receipt（证明 n 有已知的因式分解）
    // 2. 计算 c = x^e mod n
    // 3. 将 (n, e, c) 提交到 journal
    let receipt = default_prover()
        .prove(env, EXPONENTIATE_ELF)
        .unwrap()
        .receipt;

    // 第四步：验证 receipt
    // 任何收到这个 exponentiation receipt 的人都可以确信：
    // A) journal 中包含的模数 n 有已知的因式分解（通过组合验证了 multiply receipt）
    // B) 数字 c 是某个已知秘密 x 的模幂运算结果：x^e mod n
    //
    // 这两个陈述通过组合（composition）功能在单个 receipt 中得到证明
    receipt.verify(EXPONENTIATE_ID).unwrap();

    // 第五步：从 receipt 的 journal 中解码出结果 (n, e, c = x^e mod n)
    let (n, e, c): (u64, u64, u64) = receipt.journal.decode().unwrap();

    println!("{c} is the result of exponentiation by {e} under composite {n} with known factors");
}
