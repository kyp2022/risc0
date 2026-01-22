# 证明组合（Composition）示例

本示例演示了证明组合（proof composition）功能的基本用法。
它基于 [hello world 示例](../hello-world)，其中 [guest 程序](https://dev.risczero.com/terminology#guest-program)验证了证明者知道一个合数的因式分解。

要使用证明组合功能，你需要在 [host](https://dev.risczero.com/terminology#host) 端实现中使用 `add_assumption()`（参见 [`src/main.rs`] 文件），
在 [guest](https://dev.risczero.com/terminology#guest-program) 端使用 `env::verify()`（参见 [`methods/guest/src/main.rs`]）。

## 快速开始

首先，按照 [示例指南](https://dev.risczero.com/api/zkvm/examples/#running-the-examples) 安装依赖并检查示例的正确版本。

然后，运行示例：

```bash
cargo run --release
```

## 使用场景

组合（Composition）是 zkVM 的一个功能，支持在 guest 程序中验证 RISC Zero [receipts](https://dev.risczero.com/terminology#receipt)。
通过这个功能，多个 zkVM 程序可以被*组合*在一起，产生一个单一的 receipt，用于验证达到最终结果的所有计算。

组合功能通过 [guest 程序](https://dev.risczero.com/terminology#guest-program)中的 [`env::verify`] 方法实现。
之前证明的结果可以作为该方法的输入，允许将另一个 guest 的验证结果用于新的计算中。

在本示例中，guest 程序接受三个数字作为输入：`n`、`e` 和 `x`。
它首先使用 [`env::verify`] 来验证 `n` 有已知的因式分解。
这个函数调用在逻辑上等价于验证来自 [hello world 示例](../hello-world) 的 multiply guest 的 receipt。
然后它计算模幂运算 `c = x ^ e mod n`，并将 `n`、`e` 和 `c` 提交到 journal。

这个示例类似于可验证的 RSA 加密。
验证 `n` 有已知的因式分解类似于验证 `n` 是一个有效的 RSA 公钥模数。
`n` 和 `e` 一起构成一个"RSA 公钥"，并且保证存在一个已知的私钥。
计算 `c = x ^ e mod n` 类似于使用公钥 `(n, e)` 加密秘密 `x`，得到密文 `c`。
通过在 zkVM 中计算这个"加密"，我们产生一个单一的 receipt，它既验证了 `c` 是一个有效的密文，又证明了存在某个持有私钥的方可以解密它。

### 示例

组合功能的一些使用场景包括：

- **将程序拆分为多个部分，由不同方证明，以保护每方的隐私和数据所有权**
  - 例如：生成一个证明，证明密文是对某个有效公钥的某个值的正确加密。
  - 例如：通过连接来自每个私有分片的查询 receipt，为数据库查询生成证明。
- **将多个证明聚合为一个，以实现高效的批量验证**
  - 例如：为一个交易区块生成证明，其中每个交易本身都由一个 receipt 验证。
- **为可能拆分为多个不同操作的工作流创建单一 receipt**
  - 例如：为图像处理管道的结果生成单一 receipt，其中不同的过滤器在各自的 guest 中。

## 代码说明

### Host 端（`src/main.rs`）

1. **生成第一个证明**：调用 `multiply(17, 23)` 生成一个 receipt，证明知道 391 的因式分解
2. **设置执行环境**：使用 `add_assumption()` 将 multiply receipt 添加到环境中
3. **生成组合证明**：执行 exponentiate guest 程序，它会验证 multiply receipt 并计算模幂运算
4. **验证最终 receipt**：验证组合后的 receipt，它同时证明了因式分解和模幂运算

### Guest 端（`methods/guest/src/main.rs`）

1. **读取输入**：从 host 读取 `(n, e, x)`
2. **验证 receipt**：使用 `env::verify()` 验证 n 有已知的因式分解
3. **计算并提交**：计算 `c = x^e mod n` 并将结果提交到 journal

## 技术细节

- **证明组合**：允许在一个 guest 程序中验证另一个程序的 receipt
- **快速幂算法**：使用平方求幂法实现高效的模幂运算
- **隐私保护**：秘密值 `x` 在 guest 中保持私密，只有结果 `c` 被公开

[`env::verify`]: https://docs.rs/risc0-zkvm/*/risc0_zkvm/guest/env/fn.verify.html
[`methods/guest/src/main.rs`]: methods/guest/src/main.rs
[`src/main.rs`]: src/main.rs
[examples guide]: https://dev.risczero.com/api/zkvm/examples/#running-the-examples
[guest]: https://dev.risczero.com/terminology#guest-program
[guest program]: https://dev.risczero.com/terminology#guest-program
[hello world example]: ../hello-world
[host]: https://dev.risczero.com/terminology#host
[receipts]: https://dev.risczero.com/terminology#receipt

