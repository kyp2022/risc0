# RISC Zero zkVM 的 Hello World 示例

欢迎！

这个 `hello-world` 示例是 RISC Zero [zkVM] 的最小应用程序，
旨在帮助您开始构建 zkVM 应用程序。

我们推荐使用[此教程][tutorial]来逐步构建您的第一个 zkVM 应用程序。

## 快速开始

首先，请遵循 [示例指南] 安装依赖并检出正确版本的示例。

然后，使用以下命令运行示例：

```bash
cargo run --release
```

恭喜！您刚刚构建了一个零知识证明，证明您知道 391 的因子。

## 使用场景

为 RISC Zero [zkVM] 编写应用程序是软件开发人员生成零知识证明的最简单方法。
无论您是否为区块链构建，RISC Zero 都提供了最灵活和成熟的
开发涉及零知识证明（ZKP）应用程序的生态系统。

您可以在本地运行 zkVM，您的秘密永远不会离开您的机器，
或者您可以将程序和输入上传到 [Bonsai] 进行远程证明。

## 项目组织结构

zkVM 应用程序被组织成 [host程序] 和 [guest程序]。
host程序可以在 [`src/main.rs`] 中找到，guest程序可以在 [`methods/guest/src/main.rs`] 中找到。

[host] 首先 [执行] guest程序，然后 [证明执行][prove] 以构建 [receipt]。
receipt 可以传递给第三方，第三方可以检查 [journal] 以查看程序输出，
并可以 [验证] [receipt] 以确保 [guest程序] 执行的完整性。

### 证明了什么？

[receipt] 证明 [guest程序] 被正确执行，并且 `receipt.journal` 的内容
与 guest 程序执行期间由 `env::commit()` 写入的内容匹配。

通过运行演示，Alice 展示了她知道两个相乘后等于 `receipt.journal` 中写入数字的整数。
因此，Alice 证明了写入 `receipt.journal` 中的数字是合数 — 并且她知道因子 —
而不透露任何进一步的信息。

## 教程：构建您的第一个 zkVM 应用程序

我们推荐使用[此教程][tutorial]来逐步构建您的第一个 zkVM 应用程序。有关更多资料，请查看 [开发者文档]。

[`methods/guest/src/main.rs`]: ./methods/guest/src/main.rs
[`src/main.rs`]: ./src/main.rs
[Bonsai]: https://dev.bonsai.xyz
[developer docs]: https://dev.risczero.com/zkvm
[examples guide]: https://dev.risczero.com/api/zkvm/examples/#running-the-examples
[executes]: https://dev.risczero.com/terminology#execute
[guest program]: https://dev.risczero.com/terminology#guest-program
[host]: https://dev.risczero.com/terminology#host
[host program]: https://dev.risczero.com/terminology#host-program
[journal]: https://dev.risczero.com/terminology#journal
[prove]: https://dev.risczero.com/terminology#prove
[receipt]: https://dev.risczero.com/terminology#receipt
[tutorial]: https://dev.risczero.com/api/zkvm/tutorials/hello-world
[verify]: https://dev.risczero.com/terminology#verify
[zkVM]: https://dev.risczero.com/zkvm
