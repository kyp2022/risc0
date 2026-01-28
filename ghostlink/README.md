# GhostLink MVP

This is the MVP implementation of GhostLink, a client-side ZK proving system for GitHub credentials.

## 快速开始

### 1. 安装依赖

```bash
# 安装 Rust
curl https://sh.rustup.rs -sSf | sh

# 安装 RISC Zero zkVM
cargo install cargo-risczero
cargo risczero install

# 安装 Foundry（用于部署合约）
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### 2. 构建项目

```bash
cargo build --release
```

### 3. 启动后端服务

```bash
cd host
cargo run
```

服务会在 `http://localhost:3000` 启动。**请记录显示的 Image ID**，用于部署合约。

### 4. 部署合约

```bash
cd contracts

# 设置环境变量
export PRIVATE_KEY=0x你的钱包私钥
export IMAGE_ID=0x从后端服务获取的ImageID
export RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# 部署到 Sepolia
forge script Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify

# 或部署到本地测试网
forge script Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### 5. 启动前端

```bash
cd web
python3 -m http.server 8080
```

访问 `http://localhost:8080` 使用应用。

## 详细说明

请查看 [RUN_GUIDE.md](./RUN_GUIDE.md) 获取完整的运行指南。

## 项目结构

*   `methods/guest`: zkVM Guest 代码（验证逻辑）
*   `host`: 后端服务（生成证明）
*   `contracts`: Solidity 智能合约
*   `web`: 前端界面

## 技术特点

- ✅ 使用 RISC Zero 官方预编译 Verifier 合约（无需部署）
- ✅ 本地生成 Groth16 证明（无需云服务）
- ✅ 支持 Sepolia 测试网和以太坊主网
- ✅ 完全去中心化，数据隐私保护

## 注意事项

⚠️ **重要**：
- 这是一个演示项目，不应用于生产环境
- 合约未经审计，存在安全风险
- 不要在主网使用真实资金
