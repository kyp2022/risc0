# GhostLink (Zero-Knowledge Credential Prototype on RISC Zero zkVM)

English | [中文](#中文)

GhostLink is an end-to-end prototype for **local proof generation** and **on-chain verification**. It turns Web2-like user data (e.g., developer profile signals) into a verifiable proof, and can optionally mint an on-chain credential (SBT).

This repository also vendors upstream `risc0/` code and related experiments. If you only want to run GhostLink quickly, start in `ghostlink/`.

## What You Get

- **Local ZK proving**: raw data stays on your machine
- **On-chain verification**: verify proofs and mint an SBT
- **Full demo stack**: Guest (zkVM) + Host prover service + Solidity contract + Web UI

## Quick Start (Run Proof Generation Locally)

Prerequisites:
- `git-lfs` (if you pulled LFS assets)
- Rust (use `rust-toolchain.toml` where provided)
- Docker (this demo uses `risc0-zkvm` with the `docker` feature to generate Groth16 proofs)

```bash
git lfs install
git lfs pull

cd ghostlink
cargo build --release
cargo run -p ghostlink-host
```

After it starts, open `http://localhost:3000` (or check the terminal logs) and note the printed **Image ID** for contract deployment.

## Optional: Mint an SBT (Local Chain or Testnet)

Prerequisite: Foundry (`anvil`, `forge`).

Local chain (recommended first):
```bash
# Terminal 1
anvil

# Terminal 2
cd ghostlink/contracts
forge create GhostLinkSBT \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974...ff80 \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0xYOUR_IMAGE_ID)
```

Web UI:
```bash
cd ghostlink/web
python3 -m http.server 8080
```
Open `http://localhost:8080` and follow the page steps to deploy/mint.

For the complete walkthrough and troubleshooting, see `ghostlink/RUN_GUIDE.md` and `ghostlink/DEMO.md`.

## Repository Layout (High-Level)

- `ghostlink/`: GhostLink MVP (Guest/Host/contract/web UI)
- `ghostlinkV2/`: v2 exploration (design docs, scripts)
- `risc0/`: RISC Zero zkVM and related crates
- `examples/`, `benchmarks/`: examples and benchmarks
- `web/`, `website/`: upstream web/docs related components

## FAQ

- **Proving is slow / appears stuck**: first run often pulls Docker images and compiles many deps; ensure Docker is running and disk space is sufficient.
- **Contract deployment fails**: get it working on `anvil` first; testnets require a correct `RPC_URL` and enough test ETH.

## Contributing & Security

- Repo-wide conventions and commands: `CONTRIBUTING.md`
- Do not commit secrets (keys/certs/tokens). Security contact: `SECURITY.md`

---

# 中文

[English](#ghostlink-zero-knowledge-credential-prototype-on-risc-zero-zkvm) | 中文

GhostLink 是一个“本地生成证明、链上可验证”的零知识凭证原型：把 Web2 行为数据（例如开发者画像）在本地做隐私计算，生成可验证的证明，并可选择把结果铸造成链上凭证（SBT）。

本仓库同时包含上游 `risc0/` 代码与多个实验目录；如果你只想最快跑通 GhostLink，请从 `ghostlink/` 开始。

## 你能得到什么

- **本地生成零知识证明**：原始数据不必上传到服务器
- **可上链验证**：使用合约验证证明并铸造凭证
- **完整端到端示例**：Guest（zkVM）+ Host（证明服务）+ 合约 + 前端页面

## 快速开始（本地跑通证明）

前置要求：
- 已安装 `git-lfs`（如果仓库包含大文件资产）
- 已安装 Rust（建议使用 `rust-toolchain.toml` 指定版本）
- 已安装 Docker（本项目默认使用 `risc0-zkvm` 的 `docker` 特性生成 Groth16 证明）

```bash
git lfs install
git lfs pull

cd ghostlink
cargo build --release
cargo run -p ghostlink-host
```

启动成功后访问 `http://localhost:3000`（或查看终端日志），记录输出的 **Image ID**，后续部署合约会用到。

## 可选：上链铸造 SBT（本地或测试网）

前置要求：已安装 Foundry（`anvil`、`forge`）。

本地链（推荐先跑通）：
```bash
# 终端 1
anvil

# 终端 2
cd ghostlink/contracts
forge create GhostLinkSBT \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974...ff80 \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0x你的ImageID)
```

前端页面：
```bash
cd ghostlink/web
python3 -m http.server 8080
```
浏览器打开 `http://localhost:8080`，按页面步骤完成部署与铸造。

更详细的流程、参数说明与故障排查见：`ghostlink/RUN_GUIDE.md`、`ghostlink/DEMO.md`。

## 目录导览

- `ghostlink/`：GhostLink MVP（Guest/Host/合约/前端）
- `ghostlinkV2/`：第二版探索（包含设计文档与构建脚本）
- `risc0/`：RISC Zero zkVM 与相关 crates
- `examples/`、`benchmarks/`：示例与基准测试
- `web/`、`website/`：网页与文档站点（与上游组件相关）

## 常见问题

- **证明生成很慢/卡住**：首次运行通常会拉取 Docker 镜像并编译较多依赖；请确保 Docker 正常运行且磁盘空间充足。
- **合约部署失败**：优先用 `anvil` 本地链跑通；测试网需要正确的 `RPC_URL` 与足够的测试币。

## 贡献与安全

- 代码规范与仓库级命令参考：`CONTRIBUTING.md`
- 请勿提交任何私钥、证书、访问令牌等敏感信息；安全联系方式见：`SECURITY.md`
