# 问题发现方法论总结

> **目的**: 总结如何系统性地发现和修复 RISC Zero 集成中的错误  
> **更新日期**: 2026-01-26

---

## 📋 目录

1. [问题发现流程](#1-问题发现流程)
2. [具体问题分析](#2-具体问题分析)
3. [依据来源总结](#3-依据来源总结)
4. [避免错误的检查清单](#4-避免错误的检查清单)

---

## 1. 问题发现流程

### 1.1 系统性检查方法

```
1. 对比官方实现
   ↓
2. 检查接口定义
   ↓
3. 验证数据格式匹配
   ↓
4. 测试端到端流程
```

### 1.2 关键检查点

| 检查项 | 检查方法 | 依据来源 |
|--------|---------|---------|
| **接口定义** | 对比官方源码 | `risc0-ethereum/contracts/src/IRiscZeroVerifier.sol` |
| **数据格式** | 对比 Guest 和合约 | Guest 代码 vs 合约代码 |
| **哈希算法** | 查看 RISC Zero 实现 | `risc0/zkvm/src/receipt.rs` |
| **证明格式** | 对比 v1 和 v2 | 工作版本 vs 新版本 |

---

## 2. 具体问题分析

### 问题 1: 合约验证接口返回值类型错误

#### 🔍 发现过程

**步骤 1: 查看用户代码**
```solidity
// ghostlinkV2/contract/IRiscZeroVerifier.sol (原始版本)
function verify(...) external view returns (bool success);
```

**步骤 2: 对比官方实现**
```bash
# 搜索 RISC Zero 官方接口定义
grep -r "function verify" risc0-ethereum/contracts/src/
```

**步骤 3: 找到官方定义**
```solidity
// risc0-ethereum/contracts/src/IRiscZeroVerifier.sol
function verify(bytes calldata seal, bytes32 imageId, bytes32 journalDigest) external view;
// ✅ 注意：没有 returns (bool)
```

**步骤 4: 查看文档注释**
```solidity
/// @notice Verify that the given seal is a valid RISC Zero proof...
///     Reverts on failure.  // ✅ 关键：验证失败时 revert，不返回 bool
```

**步骤 5: 验证合约调用**
```solidity
// ghostlinkV2/contract/GhostLinkSBT.sol (原始版本)
bool verified = verifier.verify(seal, imageId, journalHash);  // ❌ 错误
require(verified, "ZK proof verification failed");
```

**发现**: 
- 接口定义返回 `bool`，但官方接口不返回值
- 合约中尝试读取返回值，但函数不返回任何值
- 这会导致编译错误或运行时错误

#### 📚 依据来源

1. **官方源码**: `risc0-ethereum/contracts/src/IRiscZeroVerifier.sol`
   - 第 207 行：函数签名明确不返回值
   - 第 200 行：注释说明 "Reverts on failure"

2. **官方文档注释**:
   ```solidity
   /// @dev This method additionally ensures that the input hash is all-zeros...
   ///     and there are no assumptions (i.e. the receipt is unconditional).
   ```
   说明验证失败时会 revert，而不是返回 false

3. **Solidity 最佳实践**:
   - 验证函数通常使用 revert 模式（如 OpenZeppelin 的 `require()`）
   - 返回 bool 的模式较少使用，因为需要额外的 `require()` 检查

#### ✅ 修复方法

```solidity
// 1. 修正接口定义
function verify(...) external view;  // 移除 returns (bool)

// 2. 修正调用方式
verifier.verify(seal, imageId, journalHash);  // 直接调用，失败时自动 revert
```

---

### 问题 2: Groth16 证明格式问题

#### 🔍 发现过程

**步骤 1: 用户提问**
> "你之前的方案已经是可以了。现在我写了第二代的代码，但是我不知道这里是否是Groth16的格式"

**步骤 2: 对比 v1 和 v2 的配置**
```toml
# ghostlink (v1) - 工作版本
risc0-zkvm = { version = "3.0.0", features = ["docker"] }

# ghostlinkV2 - 新版本
risc0-zkvm = { workspace = true, features = ["client", "std"] }
```

**步骤 3: 检查 Host 代码**
```rust
// 两个版本都使用了
prover.compress(&ProverOpts::groth16(), &receipt);
```

**步骤 4: 发现差异**
- v1 有 `docker` feature
- v2 缺少 `docker` feature
- 缺少 `docker` feature 可能导致无法生成 Groth16 证明

**步骤 5: 验证 Docker Feature 的作用**
```rust
// 查看 RISC Zero 文档和代码
// docker feature 用于通过 Docker 容器执行 Groth16 压缩
```

#### 📚 依据来源

1. **代码对比**: 
   - v1（工作版本）使用了 `docker` feature
   - v2（新版本）没有使用 `docker` feature

2. **RISC Zero 文档**:
   - Docker feature 允许通过容器执行 Groth16 压缩
   - 无需本地安装 Groth16 组件

3. **实际测试**:
   - v1 能生成 Groth16 证明
   - v2 在 Dev Mode 下生成假证明

#### ✅ 修复方法

```toml
# 添加 docker feature
risc0-zkvm = { workspace = true, features = ["client", "std", "docker"] }
```

---

### 问题 3: Journal Hash 必须使用 SHA-256

#### 🔍 发现过程

**步骤 1: 查看合约代码**
```solidity
// 用户可能使用 keccak256（以太坊常用）
bytes32 journalHash = keccak256(abi.encodePacked(...));  // ❌ 错误假设
```

**步骤 2: 查看 RISC Zero 源码实现**
```rust
// risc0/zkvm/src/receipt.rs
impl risc0_binfmt::Digestible for Journal {
    fn digest<S: Sha256>(&self) -> Digest {
        *S::hash_bytes(&self.bytes)  // ✅ 使用 SHA-256
    }
}
```

**步骤 3: 查看合约中的注释**
```solidity
// ghostlinkV2/contract/GhostLinkSBT.sol
// IMPORTANT: RISC Zero on-chain verifier requires SHA-256 for the journal hash
bytes32 journalHash = sha256(...);  // ✅ 正确
```

**步骤 4: 验证 RISC Zero 合约库**
```solidity
// risc0-ethereum/contracts/src/IRiscZeroVerifier.sol
function digest(ReceiptClaim memory claim) internal pure returns (bytes32) {
    return sha256(abi.encodePacked(...));  // ✅ 所有地方都使用 sha256
}
```

#### 📚 依据来源

1. **RISC Zero 源码**:
   - `risc0/zkvm/src/receipt.rs` 第 291 行：`fn digest<S: Sha256>`
   - 明确使用 SHA-256 trait

2. **RISC Zero 合约库**:
   - `risc0-ethereum/contracts/src/IRiscZeroVerifier.sol`
   - 所有哈希计算都使用 `sha256()`

3. **合约注释**:
   - 明确说明 "requires SHA-256"

4. **常见错误模式**:
   - 以太坊开发者习惯使用 `keccak256`
   - 但 RISC Zero 使用 SHA-256（这是设计选择）

#### ✅ 修复方法

```solidity
// ✅ 正确：使用 sha256
bytes32 journalHash = sha256(abi.encodePacked(msg.sender, nullifier, uint8(credType)));

// ❌ 错误：不要使用 keccak256
// bytes32 journalHash = keccak256(abi.encodePacked(...));
```

---

### 问题 4: 依赖配置导致构建错误

#### 🔍 发现过程

**步骤 1: 用户报告构建错误**
```
error: failed to run custom build command for `risc0-sys`
Could not build metal kernels
```

**步骤 2: 对比 v1 和 v2 的依赖配置**
```toml
# v1 - 使用 crates.io 预编译版本
risc0-zkvm = { version = "3.0.0" }

# v2 - 使用本地源码路径
risc0-zkvm = { path = "../risc0/zkvm" }
```

**步骤 3: 理解差异**
- **预编译版本**: 已包含编译好的 Metal kernels，无需本地构建
- **本地路径**: 需要从源码编译，包括 Metal kernels

**步骤 4: 检查构建要求**
- Metal kernels 需要 Xcode Command Line Tools
- macOS 上如果未安装，构建会失败

#### 📚 依据来源

1. **构建错误信息**:
   ```
   xcrun: error: unable to find utility "metal"
   ```
   明确指向 Metal 工具缺失

2. **代码对比**:
   - v1 使用预编译版本，无需构建
   - v2 使用本地路径，需要构建

3. **RISC Zero 构建系统**:
   - `risc0/sys/build.rs` 检查是否需要构建 Metal kernels
   - 预编译版本跳过构建步骤

#### ✅ 修复方法

```toml
# 使用预编译版本（与 v1 一致）
risc0-zkvm = { version = "3.0.0", default-features = false }
```

---

## 3. 依据来源总结

### 3.1 官方源码（最可靠）

**位置**: `risc0-ethereum/contracts/src/` 和 `risc0/zkvm/src/`

**用途**:
- ✅ 接口定义（函数签名）
- ✅ 数据格式（Journal digest 实现）
- ✅ 哈希算法（SHA-256 vs Keccak256）

**检查方法**:
```bash
# 搜索接口定义
grep -r "function verify" risc0-ethereum/contracts/src/

# 搜索实现细节
grep -r "sha256\|Sha256" risc0/zkvm/src/
```

### 3.2 工作版本对比（快速验证）

**位置**: `ghostlink/` (v1 工作版本)

**用途**:
- ✅ 配置对比（Cargo.toml）
- ✅ 代码模式对比（Host 代码）
- ✅ 已验证的工作方案

**检查方法**:
```bash
# 对比配置
diff ghostlink/host/Cargo.toml ghostlinkV2/host/Cargo.toml

# 对比代码
diff ghostlink/host/src/main.rs ghostlinkV2/host/src/main.rs
```

### 3.3 官方文档（理解设计）

**位置**: `website/` 和 RISC Zero 官方文档

**用途**:
- ✅ 理解设计意图
- ✅ 最佳实践
- ✅ 常见问题

**检查方法**:
- 查看官方文档网站
- 搜索 GitHub Issues
- 查看示例代码

### 3.4 实际测试（最终验证）

**方法**:
```bash
# 1. 构建测试
cargo build --release

# 2. 运行测试
cargo run

# 3. API 测试
curl -X POST http://localhost:3000/api/v1/prove ...

# 4. 合约测试
cast send $CONTRACT "mint(...)" ...
```

---

## 4. 避免错误的检查清单

### 4.1 接口定义检查

- [ ] **对比官方接口**: 检查 `risc0-ethereum/contracts/src/IRiscZeroVerifier.sol`
- [ ] **函数签名**: 参数类型、返回值类型必须完全匹配
- [ ] **文档注释**: 注意 "Reverts on failure" 等关键说明
- [ ] **调用方式**: 根据接口定义决定是否需要检查返回值

**检查命令**:
```bash
# 查看官方接口定义
cat risc0-ethereum/contracts/src/IRiscZeroVerifier.sol | grep -A 5 "function verify"
```

### 4.2 数据格式检查

- [ ] **Journal 格式**: Guest 代码输出的字节序列
- [ ] **Journal Hash**: 合约中计算的哈希值
- [ ] **哈希算法**: 必须使用 `sha256()`（不是 `keccak256()`）
- [ ] **字节对齐**: `abi.encodePacked()` 的打包方式

**检查方法**:
```rust
// Guest 代码
let mut journal = Vec::new();
journal.extend_from_slice(&address);    // 20 bytes
journal.extend_from_slice(&nullifier);  // 32 bytes
journal.push(type_id);                  // 1 byte
// 总计: 53 bytes
```

```solidity
// 合约代码
bytes32 journalHash = sha256(
    abi.encodePacked(msg.sender, nullifier, uint8(credType))
);
// 必须匹配 Guest 输出的 53 字节
```

### 4.3 证明格式检查

- [ ] **Groth16 压缩**: 使用 `ProverOpts::groth16()`
- [ ] **Docker Feature**: 确保启用了 `docker` feature
- [ ] **Seal 格式**: `[4字节选择器][192字节Groth16证明]`
- [ ] **选择器**: 从 `verifier_parameters` 的前 4 字节提取

**检查代码**:
```rust
// Host 代码
let compress_result = prover.compress(&ProverOpts::groth16(), &receipt);
let groth16_inner = groth16_receipt.inner.groth16().unwrap();
let selector = &verifier_params.as_bytes()[..4];
let mut seal_bytes = Vec::new();
seal_bytes.extend_from_slice(selector);
seal_bytes.extend_from_slice(seal_data.as_ref());
```

### 4.4 依赖配置检查

- [ ] **版本选择**: 使用预编译版本（`version = "3.0.0"`）还是本地路径（`path = "..."`）
- [ ] **Feature 启用**: 确保启用了必要的 features（如 `docker`）
- [ ] **构建要求**: 如果使用本地路径，需要满足构建要求（Xcode Command Line Tools）

**检查配置**:
```toml
# ✅ 推荐：预编译版本 + docker feature
risc0-zkvm = { version = "3.0.0", features = ["docker"] }

# ⚠️ 需要构建：本地路径
risc0-zkvm = { path = "../risc0/zkvm", features = ["docker"] }
```

---

## 5. 问题发现的关键技巧

### 5.1 对比法

**原理**: 对比工作版本和新版本，找出差异

**步骤**:
1. 找到工作版本（v1）
2. 对比关键配置和代码
3. 找出差异点
4. 分析差异是否导致问题

**示例**:
```bash
# 对比 Cargo.toml
diff ghostlink/host/Cargo.toml ghostlinkV2/host/Cargo.toml

# 对比接口定义
diff ghostlink/contracts/IRiscZeroVerifier.sol ghostlinkV2/contract/IRiscZeroVerifier.sol
```

### 5.2 源码追踪法

**原理**: 从用户代码追溯到官方实现，验证正确性

**步骤**:
1. 找到用户代码中的关键调用
2. 搜索官方源码中的对应实现
3. 对比函数签名、参数类型、返回值
4. 检查文档注释

**示例**:
```solidity
// 用户代码
verifier.verify(seal, imageId, journalHash);

// 追踪到官方源码
// risc0-ethereum/contracts/src/IRiscZeroVerifier.sol
function verify(...) external view;  // 不返回 bool
```

### 5.3 文档注释分析法

**原理**: 官方代码的注释通常包含关键信息

**关键注释模式**:
- `/// Reverts on failure` → 不返回 bool
- `/// IMPORTANT:` → 重要提示
- `/// @dev` → 实现细节
- `/// @notice` → 功能说明

**示例**:
```solidity
/// @notice Verify that the given seal is a valid RISC Zero proof...
///     Reverts on failure.  // ✅ 关键信息
function verify(...) external view;
```

### 5.4 错误信息分析法

**原理**: 构建错误和运行时错误通常指向根本原因

**错误类型**:
- **编译错误**: 类型不匹配、函数不存在
- **运行时错误**: 验证失败、哈希不匹配
- **构建错误**: 缺少工具、依赖问题

**分析步骤**:
1. 提取错误关键词
2. 搜索相关代码位置
3. 对比正确实现
4. 找出差异

---

## 6. 总结：避免错误的黄金法则

### ✅ 法则 1: 始终对比官方实现

**操作**:
```bash
# 1. 找到官方源码位置
find . -name "IRiscZeroVerifier.sol" -path "*/risc0-ethereum/*"

# 2. 对比接口定义
diff your_interface.sol official_interface.sol

# 3. 确保完全匹配
```

### ✅ 法则 2: 使用工作版本作为参考

**操作**:
- 保留一个已知工作的版本（如 v1）
- 新版本开发时，对比关键配置
- 发现差异时，分析是否必要

### ✅ 法则 3: 验证端到端流程

**操作**:
```bash
# 1. 生成证明
curl -X POST http://localhost:3000/api/v1/prove ...

# 2. 检查证明格式
# receipt_hex 应该是真实的 Groth16 证明（不是 fake_receipt）

# 3. 调用合约
cast send $CONTRACT "mint(...)" ...

# 4. 验证是否成功
```

### ✅ 法则 4: 阅读文档注释

**操作**:
- 不要只看函数签名，还要看注释
- 特别注意 `IMPORTANT`、`@dev`、`@notice` 标记
- 理解设计意图，而不只是语法

### ✅ 法则 5: 测试驱动开发

**操作**:
- 先写测试用例
- 运行测试发现错误
- 修复错误后再继续

---

## 7. 快速检查命令

### 检查接口定义
```bash
# 查看官方接口
grep -A 3 "function verify" risc0-ethereum/contracts/src/IRiscZeroVerifier.sol

# 对比用户接口
diff ghostlinkV2/contract/IRiscZeroVerifier.sol risc0-ethereum/contracts/src/IRiscZeroVerifier.sol
```

### 检查哈希算法
```bash
# 查看 RISC Zero 实现
grep -r "Sha256\|sha256" risc0/zkvm/src/receipt.rs

# 检查合约中的使用
grep -r "sha256\|keccak256" ghostlinkV2/contract/
```

### 检查配置
```bash
# 对比 Cargo.toml
diff ghostlink/host/Cargo.toml ghostlinkV2/host/Cargo.toml

# 检查 features
grep "features" ghostlinkV2/host/Cargo.toml
```

---

**文档版本**: v1.0  
**最后更新**: 2026-01-26  
**作者**: AI Assistant (基于实际调试经验总结)
