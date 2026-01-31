# GhostLink MVP (End-to-End Demo)

English | [中文](#中文)

GhostLink MVP is a runnable end-to-end demo: generate ZK proofs (Groth16) locally, and optionally submit them to a Solidity contract for verification and SBT minting.

If you want more context first:
- `GhostLink_Product_Spec.md`: product vision & use cases
- `GhostLink_Web_Technical_Design.md`: web-first architecture notes
- `RUN_GUIDE.md`: full step-by-step tutorial + troubleshooting

## Prerequisites

- Rust (use `rust-toolchain.toml` in this directory)
- Docker (`ghostlink-host` enables `risc0-zkvm` with the `docker` feature)
- Optional: Foundry (`anvil` / `forge`) for local chain or testnet deployment

## 5-Minute Run (Proof Only)

```bash
cd ghostlink
cargo build --release
cargo run -p ghostlink-host
```

The service starts at `http://localhost:3000`. The terminal prints an **Image ID** (required for contract deployment).

## Optional: Mint on a Local Chain (Recommended First)

```bash
# Terminal 1: start local chain
anvil

# Terminal 2: deploy contract (replace ImageID with the value from host logs)
cd ghostlink/contracts
forge create GhostLinkSBT \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0xYOUR_IMAGE_ID)
```

Web UI:
```bash
cd ghostlink/web
python3 -m http.server 8080
```
Open `http://localhost:8080` and follow the steps to generate proofs and mint.

## Project Structure

- `methods/guest/`: zkVM Guest (verification logic)
- `host/`: prover service (proof generation, Image ID / Journal / Nullifier)
- `contracts/`: Solidity contract (verify proof, mint SBT)
- `web/`: static web page (served via any local HTTP server)

## Notes (Read Before Publishing)

- Demo/prototype only; contracts are not audited.
- Never commit secrets (keys/certs/tokens). Always review `git status` and recent commits before pushing.
- First run may be slow due to Docker image pulls and dependency builds.

---

# 中文

[English](#ghostlink-mvp-end-to-end-demo) | 中文

GhostLink MVP 是一个可直接运行的端到端示例：在本地生成零知识证明（Groth16），并可选择把证明提交到链上合约进行验证与铸造 SBT。

如果你希望先理解产品与架构背景，建议从以下文档开始：
- `GhostLink_Product_Spec.md`：产品与场景
- `GhostLink_Web_Technical_Design.md`：Web 侧架构设计
- `RUN_GUIDE.md`：最完整的运行教程与排错

## 你需要准备什么

- Rust（建议使用本目录 `rust-toolchain.toml` 指定版本）
- Docker（本项目 `ghostlink-host` 默认启用 `risc0-zkvm` 的 `docker` 特性）
- 可选：Foundry（`anvil` / `forge`，用于本地链或测试网部署合约）

## 5 分钟跑通（只跑证明）

```bash
cd ghostlink
cargo build --release
cargo run -p ghostlink-host
```

服务默认在 `http://localhost:3000` 启动；终端会打印 **Image ID**（后续部署合约需要）。

## 可选：本地链上铸造（推荐先走本地链）

```bash
# 终端 1：启动本地链
anvil

# 终端 2：部署合约（把 ImageID 换成你启动 host 时看到的值）
cd ghostlink/contracts
forge create GhostLinkSBT \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0x你的ImageID)
```

前端页面：
```bash
cd ghostlink/web
python3 -m http.server 8080
```
打开 `http://localhost:8080`，按页面步骤完成生成证明与铸造。

## 目录结构

- `methods/guest/`：zkVM Guest（验证逻辑）
- `host/`：证明服务（生成证明、输出 Image ID / Journal / Nullifier）
- `contracts/`：Solidity 合约（验证证明、铸造 SBT）
- `web/`：静态前端页面（本地 http server 即可运行）

## 注意事项（强烈建议先读）

- 这是演示与原型代码，不建议直接用于生产环境；合约未经审计。
- 不要把任何私钥、证书、访问令牌提交到公开仓库；发布前请自查 `git status` 与历史提交。
- 证明生成可能较慢；首次运行会拉取 Docker 镜像并进行较多编译工作。
