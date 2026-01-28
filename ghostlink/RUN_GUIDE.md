# GhostLink 运行指南

## 项目概述

GhostLink 是一个基于 RISC Zero zkVM 的零知识证明系统，用于将 Web2 数据（如 GitHub）转换为链上凭证（SBT）。

## 前置要求

### 1. 安装 Rust 工具链

```bash
# 安装 Rust
curl https://sh.rustup.rs -sSf | sh

# 安装 RISC Zero zkVM (v3.0.0)
cargo install cargo-risczero
cargo risczero install
```

### 2. 安装 Foundry（用于部署合约）

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### 3. 准备钱包

- 准备一个以太坊钱包（MetaMask 等）
- 确保钱包有足够的测试网 ETH（Sepolia）
- 获取钱包私钥（用于部署合约）

## 运行步骤

### 步骤 1: 构建项目

```bash
cd risc0/ghostlink

# 构建 Guest 代码和 Host 服务
cargo build --release
```

### 步骤 2: 启动后端服务（生成证明）

```bash
cd host
cargo run
```

后端服务会在 `http://localhost:3000` 启动。

**重要提示**：
- 首次运行会下载 Docker 镜像（用于 Groth16 证明生成）
- 生成证明可能需要几分钟时间
- 服务启动后会显示 Guest Image ID，请记录下来用于部署合约

### 步骤 3: 部署智能合约

#### 方式 A: 部署到 Sepolia 测试网

```bash
cd contracts

# 设置环境变量
export ETH_WALLET_PRIVATE_KEY=0x你的钱包私钥
export RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# 部署合约
forge script --rpc-url $RPC_URL --broadcast --verify \
  --constructor-args $(cast to-bytes32 0x你的ImageID) \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  Deploy.s.sol

# 或者手动部署
forge create GhostLinkSBT \
  --rpc-url $RPC_URL \
  --private-key $ETH_WALLET_PRIVATE_KEY \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0x你的ImageID)
```

#### 方式 B: 部署到本地测试网（Anvil）

```bash
# 终端 1: 启动本地测试网
anvil

# 终端 2: 部署合约
cd contracts
forge create GhostLinkSBT \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --constructor-args 0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
  --constructor-args $(cast to-bytes32 0x你的ImageID)
```

**合约构造函数参数**：
1. `_verifierAddress`: RISC Zero 官方预编译 Verifier 地址
   - Sepolia: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`
   - Mainnet: `0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`
2. `_imageId`: Guest 代码的 Image ID（从后端服务启动日志中获取）

### 步骤 4: 启动前端

```bash
cd web

# 使用简单的 HTTP 服务器（Python）
python3 -m http.server 8080

# 或使用 Node.js
npx http-server -p 8080

# 或使用其他 HTTP 服务器
```

### 步骤 5: 使用应用

1. 打开浏览器访问 `http://localhost:8080`
2. 连接钱包（MetaMask）
3. 在 "Step B" 中输入 Image ID（从后端服务日志中获取）
4. 点击 "Deploy SBT" 部署合约
5. 在 "Step 3" 中输入 GitHub 用户 JSON 数据，点击 "Generate Proof"
6. 等待证明生成完成（可能需要几分钟）
7. 在 "Step 4" 中输入合约地址，点击 "Mint SBT"

## 测试数据示例

```json
{
  "id": 12345,
  "login": "test_user",
  "created_at": "2020-01-01T00:00:00Z",
  "public_repos": 5
}
```

**验证规则**：
- `public_repos > 0`（必须有公开仓库）
- `created_at` 年份 < 2023（账号必须创建于 2023 年之前）

## 项目结构

```
ghostlink/
├── contracts/          # Solidity 智能合约
│   └── GhostLinkSBT.sol
├── methods/            # zkVM Guest 代码
│   └── guest/src/main.rs
├── host/               # 后端服务（生成证明）
│   └── src/main.rs
└── web/                # 前端界面
    └── index.html
```

## 技术说明

### Verifier 合约

项目使用 RISC Zero 官方预编译的 `RiscZeroVerifierRouter` 合约：
- **Sepolia**: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`
- **Mainnet**: `0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`

这个合约会自动路由到正确的 verifier 实现，支持多个版本的 zkVM。

### Journal 格式

Guest 代码输出的 Journal 格式：
```
[12 bytes padding][20 bytes address][32 bytes nullifier] = 64 bytes
```

合约会计算 `sha256(journal)` 作为 `postStateDigest` 进行验证。

### 证明生成

- 使用 RISC Zero zkVM v3.0.0
- 生成 Groth16 SNARK 证明（用于链上验证）
- 证明生成在本地完成，不需要云服务

## 故障排查

### 1. 后端服务无法启动

- 检查 Docker 是否运行
- 检查端口 3000 是否被占用
- 查看错误日志

### 2. 证明生成失败

- 检查输入 JSON 格式是否正确
- 检查验证规则是否满足（public_repos > 0, created_at < 2023）
- 查看后端服务日志

### 3. 合约部署失败

- 检查钱包是否有足够的 ETH
- 检查 RPC URL 是否正确
- 检查 Image ID 格式是否正确（32 字节 hex）

### 4. Mint 失败

- 检查合约地址是否正确
- 检查钱包是否连接
- 检查 gas limit 是否足够（建议 1000000）
- 检查 nullifier 是否已被使用（每个证明只能使用一次）

## 相关链接

- [RISC Zero 文档](https://dev.risczero.com/api)
- [RISC Zero Verifier 合约地址](https://dev.risczero.com/api/blockchain-integration/contracts/verifier)
- [Foundry 文档](https://book.getfoundry.sh/)
