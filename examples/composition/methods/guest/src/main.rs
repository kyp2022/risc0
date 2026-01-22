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

use hello_world_methods::MULTIPLY_ID;
use risc0_zkvm::{guest::env, serde};

fn main() {
    // 从 host 读取输入数据
    // n: 公开的模数（合数）
    // e: 公开的指数
    // x: 秘密值（将被加密，在 guest 中保持私密）
    let (n, e, x): (u64, u64, u64) = env::read();

    // 验证 n 有已知的因式分解
    // env::verify 是证明组合（proof composition）的核心功能
    // 它验证一个之前生成的 receipt（通过 MULTIPLY_ID 标识）
    // 这里验证的是 hello-world 示例中的 multiply receipt，证明 n 是两个已知因子的乘积
    // 第二个参数是要验证的 journal 数据，这里传入 n 来验证它确实是被证明过的合数
    env::verify(MULTIPLY_ID, &serde::to_vec(&n).unwrap()).unwrap();

    // 计算模幂运算 c = x^e mod n，并将结果提交到 journal
    // 提交的数据包括：(n, e, c)
    // 这类似于 RSA 加密：使用公钥 (n, e) 加密秘密 x，得到密文 c
    env::commit(&(n, e, pow_mod(x, e, n)));
}

/// 计算模幂运算 x^e (mod n)
/// 
/// 使用快速幂算法（平方求幂法）实现，时间复杂度为 O(log e)
/// 参考：https://en.wikipedia.org/wiki/Exponentiation_by_squaring
pub fn pow_mod(x: u64, mut e: u64, n: u64) -> u64 {
    // 转换为 u128 以避免中间计算溢出
    let mut x = x as u128;
    let n = n as u128;
    let mut z = 1u128; // 结果初始化为 1

    // 快速幂算法：通过平方和乘法来计算 x^e mod n
    // 算法原理：将指数 e 表示为二进制，然后通过平方和乘法组合
    while e > 0 {
        // 如果当前位是 1，则将 x 乘到结果中
        if e % 2 == 1 {
            z = (z * x) % n
        }
        // 指数右移一位（相当于除以 2）
        e >>= 1;
        // x 平方（为下一次迭代准备）
        x = (x * x) % n;
    }
    return z as u64;
}
