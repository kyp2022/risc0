# GhostLink V2 启动和使用指南

> **版本**: v2.0  
> **更新日期**: 2026-01-26

本文档提供 GhostLink V2 项目的完整启动流程和合约调用指南。

---

## 📋 目录

1. [环境准备](#1-环境准备)
2. [启动后端服务](#2-启动后端服务)
3. [部署智能合约](#3-部署智能合约)
4. [生成零知识证明](#4-生成零知识证明)
5. [调用合约 Mint SBT](#5-调用合约-mint-sbt)
6. [参数说明](#6-参数说明)
7. [完整示例](#7-完整示例)

---

## 1. 环境准备

### 1.1 安装 Rust 工具链

```bash
# 安装 Rust（如果未安装）
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env

# 安装 RISC Zero 工具链
cargo install cargo-risczero
cargo risczero install
```

### 1.1.1 macOS 特殊要求

**重要**: 在 macOS 上构建 `risc0-sys` 需要 Xcode Command Line Tools：

```bash
# 检查是否已安装
xcode-select -p

# 如果未安装，执行：
xcode-select --install

# 等待安装完成（可能需要几分钟）
```

如果遇到 `risc0-sys` 构建错误，请参考 [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) 获取详细解决方案。

### 1.2 安装 Groth16 组件（可选，用于生产环境）

```bash
# 安装 risc0-groth16 组件（用于压缩证明）
rzup install risc0-groth16
```

**注意**: 
- 开发环境可以使用 `RISC0_DEV_MODE`，无需 Groth16
- 生产环境必须安装 Groth16 组件

### 1.3 安装 Foundry（用于部署合约）

```bash
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

---

## 2. 启动后端服务

### 2.1 构建项目

```bash
cd /Users/ppg/Desktop/zkvm/risc0/ghostlinkV2

# 构建项目（包含 Guest 代码）
cargo build --release
```

### 2.2 启动服务

#### 方式 A: 开发模式（快速测试）

```bash
cd host
RISC0_DEV_MODE=1 cargo run
```

**特点**:
- ✅ 证明生成速度快（秒级）
- ✅ 不需要 Groth16 组件
- ✅ 不需要构建 Metal kernels（避免 macOS 构建问题）
- ⚠️ 生成的证明是假的，不能用于链上验证

**注意**: 如果遇到 `risc0-sys` 构建错误，开发模式是最简单的解决方案。

#### 方式 B: 生产模式（真实证明）

```bash
cd host
cargo run --release --features prove
```

**特点**:
- ✅ 生成真实的 Groth16 证明
- ⚠️ 需要安装 `risc0-groth16` 组件
- ⚠️ 证明生成时间：几分钟（取决于硬件）

### 2.3 验证服务启动

服务启动后会显示：

```
==================================================
🔐 GhostLink V2 ZK Prover Service
==================================================
📋 Supported credentials: github, alipay, twitter, wallet
🔑 Guest Image ID: 0x6ebc99262cf7c06dad69ba031a3373b5e250cf2401f10a9bb16ca23ede40c041
==================================================
🚀 Server running at http://127.0.0.1:3000
```

**重要**: 请记录显示的 **Image ID**，部署合约时需要用到！

### 2.4 测试服务健康状态

```bash
curl http://localhost:3000/health
```

预期响应：
```json
{
  "status": "healthy",
  "version": "2.0.0"
}
```

---

## 3. 部署智能合约

### 3.1 准备部署参数

部署合约需要以下参数：

| 参数 | 说明 | 来源 |
|------|------|------|
| `_verifier` | RISC Zero Verifier Router 地址 | 官方预部署地址（见下方） |
| `_imageId` | Guest 程序 Image ID | 从后端服务启动日志获取 |
| `_baseURI` | Token 元数据基础 URI | 自定义（如：`https://api.ghostlink.io/metadata/`） |
| `_name` | Token 名称 | 自定义（如：`GhostLink SBT`） |
| `_symbol` | Token 符号 | 自定义（如：`GLSBT`） |

**RISC Zero Verifier 地址**:
- **Sepolia 测试网**: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`
- **以太坊主网**: `0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`

### 3.2 部署到 Sepolia 测试网

```bash
cd contract

# 设置环境变量
export PRIVATE_KEY=0x你的钱包私钥
export RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
export IMAGE_ID=0x6ebc99262cf7c06dad69ba031a3373b5e250cf2401f10a9bb16ca23ede40c041  # 替换为你的 Image ID

# 部署合约
forge create GhostLinkSBT \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args \
    $VERIFIER_ADDRESS \
    $IMAGE_ID \
    "https://api.ghostlink.io/metadata/" \
    "GhostLink SBT" \
    "GLSBT"
```

**或者使用部署脚本**（推荐）：

创建 `scripts/Deploy.s.sol`:

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../contract/GhostLinkSBT.sol";

contract DeployScript is Script {
    function run() external {
        address verifier = vm.envAddress("VERIFIER_ADDRESS");
        bytes32 imageId = vm.envBytes32("IMAGE_ID");
        string memory baseURI = vm.envString("BASE_URI");
        
        vm.startBroadcast();
        
        GhostLinkSBT sbt = new GhostLinkSBT(
            verifier,
            imageId,
            baseURI,
            "GhostLink SBT",
            "GLSBT"
        );
        
        console.log("Contract deployed at:", address(sbt));
        console.log("Image ID:", vm.toString(imageId));
        
        vm.stopBroadcast();
    }
}
```

然后运行：

```bash
forge script scripts/Deploy.s.sol \
  --rpc-url $RPC_URL \
  --broadcast \
  --verify \
  --sig "run()"
```

### 3.3 记录合约地址

部署成功后，记录合约地址，后续调用 `mint` 函数时需要用到。

---

## 4. 生成零知识证明

### 4.1 API 端点

```
POST http://localhost:3000/api/v1/prove
Content-Type: application/json
```

### 4.2 请求格式

```json
{
  "credential_type": "github | alipay | twitter | wallet",
  "data": {
    // 凭证类型特定的数据（见下方）
  },
  "recipient": "0x1234567890123456789012345678901234567890"
}
```

### 4.3 各凭证类型的请求示例

#### GitHub 凭证

```bash
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "github",
    "data": {
      "user_id": 12345678,
      "username": "ghostlink-user",
      "created_at": "2020-01-01T00:00:00Z",
      "public_repos": 5
    },
    "recipient": "0x1234567890123456789012345678901234567890"
  }'
```

#### 支付宝凭证

```bash
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "alipay",
    "data": {
      "balance": "15975.01",
      "id_number_hash": "0x7a8b9c1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
      "threshold": "10000"
    },
    "recipient": "0x1234567890123456789012345678901234567890"
  }'
```

#### Twitter 凭证

```bash
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "twitter",
    "data": {
      "user_id": "987654321",
      "handle": "ghostlink",
      "created_at": "2015-01-01T00:00:00Z",
      "followers_count": 100
    },
    "recipient": "0x1234567890123456789012345678901234567890"
  }'
```

#### 钱包凭证

```bash
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "wallet",
    "data": {
      "address": "0x1234567890123456789012345678901234567890",
      "balance_wei": "1000000000000000000",
      "transaction_count": 42,
      "chain_id": 11155111,
      "signature": "0xabc123...",
      "message": "GhostLink Asset-Pass Verification..."
    },
    "recipient": "0x1234567890123456789012345678901234567890"
  }'
```

### 4.4 响应格式

**成功响应**:

```json
{
  "status": "success",
  "receipt_hex": "0x1234abcd...",      // Groth16 证明（Seal）
  "journal_hex": "1234567890abcdef...", // Journal 数据（53字节）
  "image_id_hex": "6ebc99262cf7c06d...", // Image ID
  "nullifier_hex": "7a8b9c1234567890..."  // Nullifier（32字节）
}
```

**错误响应**:

```json
{
  "status": "error",
  "error_code": "INVALID_DATA | THRESHOLD_NOT_MET | ...",
  "message": "描述信息"
}
```

---

## 5. 调用合约 Mint SBT

### 5.1 函数签名

```solidity
function mint(
    bytes calldata seal,       // ZK 证明 Seal
    bytes32 nullifier,         // 防重放标识符
    CredentialType credType    // 凭证类型 (0-3)
) external returns (uint256 tokenId)
```

### 5.2 参数说明

| 参数 | 类型 | 说明 | 来源 |
|------|------|------|------|
| `seal` | `bytes` | Groth16 证明数据 | API 响应的 `receipt_hex` |
| `nullifier` | `bytes32` | 防重放标识符 | API 响应的 `nullifier_hex` |
| `credType` | `uint8` | 凭证类型枚举值 | 见下方映射表 |

**凭证类型映射**:

| 凭证类型 | 枚举值 | 说明 |
|----------|--------|------|
| GitHub | `0` | `CredentialType.GITHUB` |
| Alipay | `1` | `CredentialType.ALIPAY` |
| Twitter | `2` | `CredentialType.TWITTER` |
| Wallet | `3` | `CredentialType.WALLET` |

### 5.3 使用 Ethers.js 调用示例

```javascript
const { ethers } = require("ethers");

// 1. 连接钱包和合约
const provider = new ethers.JsonRpcProvider("https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY");
const wallet = new ethers.Wallet("YOUR_PRIVATE_KEY", provider);
const contract = new ethers.Contract(
  "0x合约地址",
  [
    "function mint(bytes calldata seal, bytes32 nullifier, uint8 credType) external returns (uint256)",
    "function imageId() external view returns (bytes32)",
  ],
  wallet
);

// 2. 准备参数（从 API 响应获取）
const apiResponse = {
  receipt_hex: "0x1234abcd...",
  nullifier_hex: "7a8b9c1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  image_id_hex: "6ebc99262cf7c06dad69ba031a3373b5e250cf2401f10a9bb16ca23ede40c041"
};

// 3. 验证 Image ID 匹配
const contractImageId = await contract.imageId();
const apiImageId = "0x" + apiResponse.image_id_hex;
if (contractImageId.toLowerCase() !== apiImageId.toLowerCase()) {
  throw new Error("Image ID mismatch!");
}

// 4. 准备 mint 参数
const seal = apiResponse.receipt_hex;  // 已经是 0x 开头的 hex 字符串
const nullifier = "0x" + apiResponse.nullifier_hex;  // 添加 0x 前缀
const credType = 0;  // GitHub = 0, Alipay = 1, Twitter = 2, Wallet = 3

// 5. 调用 mint 函数
try {
  const tx = await contract.mint(seal, nullifier, credType, {
    gasLimit: 500000  // 建议设置足够的 gas limit
  });
  
  console.log("Transaction sent:", tx.hash);
  
  const receipt = await tx.wait();
  console.log("Mint successful! Token ID:", receipt.logs[0].args.tokenId);
} catch (error) {
  console.error("Mint failed:", error.message);
}
```

### 5.4 使用 Foundry Cast 调用示例

```bash
# 设置环境变量
export CONTRACT_ADDRESS=0x你的合约地址
export PRIVATE_KEY=0x你的私钥
export RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# 从 API 响应获取参数
export SEAL=0x1234abcd...  # receipt_hex
export NULLIFIER=0x7a8b9c1234567890abcdef1234567890abcdef1234567890abcdef1234567890
export CRED_TYPE=0  # 0=GitHub, 1=Alipay, 2=Twitter, 3=Wallet

# 调用 mint
cast send $CONTRACT_ADDRESS \
  "mint(bytes,bytes32,uint8)" \
  $SEAL \
  $NULLIFIER \
  $CRED_TYPE \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --gas-limit 500000
```

---

## 6. 参数说明

### 6.1 API 响应参数详解

#### `receipt_hex` (Seal)

- **格式**: `0x` + hex 字符串
- **内容**: `[4字节选择器][192字节Groth16证明]`
- **用途**: 链上验证的零知识证明
- **来源**: Host 服务生成

**示例**:
```
0x12345678abcdeffedcba9876543210...
├─ 12345678 (4字节选择器)
└─ abcdeffedcba9876543210... (Groth16证明)
```

#### `journal_hex`

- **格式**: hex 字符串（无 `0x` 前缀）
- **内容**: `[20字节address][32字节nullifier][1字节type]` = 53字节
- **用途**: 公开输出数据
- **来源**: Guest 代码生成

**示例**:
```
1234567890123456789012345678901234567890  # 20字节 address
7a8b9c1234567890abcdef1234567890abcdef1234567890abcdef1234567890  # 32字节 nullifier
00  # 1字节 type (0=GitHub)
```

#### `image_id_hex`

- **格式**: hex 字符串（无 `0x` 前缀）
- **内容**: 32字节的 Image ID
- **用途**: 标识 Guest 程序版本
- **来源**: Guest 代码编译时生成

**验证**: 必须与合约中的 `imageId` 完全匹配

#### `nullifier_hex`

- **格式**: hex 字符串（无 `0x` 前缀）
- **内容**: 32字节的哈希值
- **用途**: 防止同一凭证重复铸造
- **来源**: Guest 代码根据凭证数据生成

**生成规则**:
- GitHub: `keccak256("github" || user_id)`
- Alipay: `keccak256("alipay" || id_number_hash)`
- Twitter: `keccak256("twitter" || user_id)`
- Wallet: `keccak256("wallet" || address || chain_id)`

### 6.2 合约 Mint 参数详解

#### `seal` (bytes)

- **类型**: `bytes calldata`
- **格式**: 完整的 `receipt_hex`（包含 `0x` 前缀）
- **验证**: 合约会调用 `verifier.verify(seal, imageId, journalHash)`

**注意事项**:
- 必须是 Groth16 格式（生产环境）
- Dev Mode 生成的假证明不能用于链上验证

#### `nullifier` (bytes32)

- **类型**: `bytes32`
- **格式**: `0x` + `nullifier_hex`
- **验证**: 合约会检查 `nullifiers[nullifier]` 是否已使用

**注意事项**:
- 每个 nullifier 只能使用一次
- 如果已使用，交易会 revert

#### `credType` (uint8)

- **类型**: `CredentialType` enum (实际是 `uint8`)
- **值**: `0` (GitHub), `1` (Alipay), `2` (Twitter), `3` (Wallet)
- **用途**: 标识凭证类型

---

## 7. 完整示例

### 7.1 完整流程：GitHub 凭证

```bash
# ========== 步骤 1: 启动后端服务 ==========
cd /Users/ppg/Desktop/zkvm/risc0/ghostlinkV2/host
RISC0_DEV_MODE=1 cargo run

# 记录 Image ID（从启动日志）
# Image ID: 0x6ebc99262cf7c06dad69ba031a3373b5e250cf2401f10a9bb16ca23ede40c041

# ========== 步骤 2: 生成证明 ==========
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "github",
    "data": {
      "user_id": 12345678,
      "username": "ghostlink-user",
      "created_at": "2020-01-01T00:00:00Z",
      "public_repos": 5
    },
    "recipient": "0x1234567890123456789012345678901234567890"
  }' > response.json

# 查看响应
cat response.json
# {
#   "status": "success",
#   "receipt_hex": "0x1234abcd...",
#   "journal_hex": "1234567890abcdef...",
#   "image_id_hex": "6ebc99262cf7c06d...",
#   "nullifier_hex": "7a8b9c1234567890..."
# }

# ========== 步骤 3: 部署合约（如果未部署）==========
cd ../contract
forge create GhostLinkSBT \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --constructor-args \
    0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187 \
    0x6ebc99262cf7c06dad69ba031a3373b5e250cf2401f10a9bb16ca23ede40c041 \
    "https://api.ghostlink.io/metadata/" \
    "GhostLink SBT" \
    "GLSBT"

# ========== 步骤 4: 调用合约 Mint ==========
# 从 response.json 提取参数
SEAL=$(cat response.json | jq -r '.receipt_hex')
NULLIFIER="0x$(cat response.json | jq -r '.nullifier_hex')"
CRED_TYPE=0  # GitHub

cast send $CONTRACT_ADDRESS \
  "mint(bytes,bytes32,uint8)" \
  $SEAL \
  $NULLIFIER \
  $CRED_TYPE \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --gas-limit 500000
```

### 7.2 使用 JavaScript/TypeScript 完整示例

```typescript
import { ethers } from "ethers";

async function mintGitHubCredential() {
  // 1. 生成证明
  const proveResponse = await fetch("http://localhost:3000/api/v1/prove", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      credential_type: "github",
      data: {
        user_id: 12345678,
        username: "ghostlink-user",
        created_at: "2020-01-01T00:00:00Z",
        public_repos: 5,
      },
      recipient: "0x1234567890123456789012345678901234567890",
    }),
  });

  const proof = await proveResponse.json();
  if (proof.status !== "success") {
    throw new Error(`Proof generation failed: ${proof.message}`);
  }

  // 2. 连接合约
  const provider = new ethers.JsonRpcProvider("https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY");
  const wallet = new ethers.Wallet("YOUR_PRIVATE_KEY", provider);
  const contract = new ethers.Contract(
    "0x合约地址",
    [
      "function mint(bytes calldata seal, bytes32 nullifier, uint8 credType) external returns (uint256)",
      "function imageId() external view returns (bytes32)",
    ],
    wallet
  );

  // 3. 验证 Image ID
  const contractImageId = await contract.imageId();
  const apiImageId = "0x" + proof.image_id_hex;
  if (contractImageId.toLowerCase() !== apiImageId.toLowerCase()) {
    throw new Error("Image ID mismatch!");
  }

  // 4. 准备参数
  const seal = proof.receipt_hex;  // 已经是 0x 开头
  const nullifier = "0x" + proof.nullifier_hex;
  const credType = 0;  // GitHub

  // 5. Mint
  const tx = await contract.mint(seal, nullifier, credType, {
    gasLimit: 500000,
  });

  console.log("Transaction sent:", tx.hash);
  const receipt = await tx.wait();
  console.log("Mint successful! Token ID:", receipt.logs[0].args.tokenId);
}

mintGitHubCredential().catch(console.error);
```

---

## 8. 故障排查

### 8.1 常见错误

#### Image ID 不匹配

**错误信息**: `VerificationFailed` 或交易 revert

**原因**: 合约部署时使用的 Image ID 与当前 Guest 代码的 Image ID 不一致

**解决**:
1. 检查后端服务启动日志中的 Image ID
2. 重新部署合约或更新合约的 Image ID（如果合约支持）

#### Nullifier 已使用

**错误信息**: `Already minted`

**原因**: 同一个 nullifier 被重复使用

**解决**: 每个凭证只能铸造一次，如需再次铸造，需要使用不同的凭证数据

#### 证明格式错误

**错误信息**: `SelectorMismatch` 或 `VerificationFailed`

**原因**: 
- Dev Mode 生成的假证明不能用于链上验证
- Seal 格式不正确

**解决**:
1. 确保使用生产模式生成证明（禁用 `RISC0_DEV_MODE`）
2. 确保安装了 `risc0-groth16` 组件

### 8.2 调试技巧

#### 检查 Journal Hash

```solidity
// 在合约中添加调试函数
function calculateJournalHash(
    address user,
    bytes32 nullifier,
    CredentialType credType
) public pure returns (bytes32) {
    return sha256(abi.encodePacked(user, nullifier, uint8(credType)));
}
```

#### 验证 Nullifier 状态

```solidity
// 检查 nullifier 是否已使用
bool used = contract.nullifiers(nullifier);
```

---

## 9. 参考资源

- [RISC Zero 官方文档](https://dev.risczero.com/)
- [RISC Zero Verifier 合约地址](https://dev.risczero.com/api/blockchain-integration/contracts/verifier)
- [Foundry 文档](https://book.getfoundry.sh/)
- [Ethers.js 文档](https://docs.ethers.org/)

---

## 10. 注意事项

⚠️ **重要提醒**:

1. **Dev Mode**: 开发模式下生成的证明不能用于链上验证
2. **Image ID**: 每次修改 Guest 代码后，Image ID 会改变，需要重新部署合约
3. **Gas Limit**: Mint 操作建议设置 `gasLimit: 500000` 或更高
4. **Nullifier**: 每个凭证只能铸造一次，请妥善保管证明数据
5. **生产环境**: 必须使用生产模式生成真实的 Groth16 证明

---

**文档版本**: v2.0  
**最后更新**: 2026-01-26
