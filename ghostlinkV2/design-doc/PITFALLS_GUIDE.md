# GhostLink V2 避坑指南

> **版本**: v2.0
> **更新日期**: 2026-01-26
> **目的**: 记录开发过程中遇到的关键问题和解决方案

---

## 📋 目录

1. [依赖配置：本地路径 vs 预编译版本](#1-依赖配置本地路径-vs-预编译版本)
2. [Groth16 证明生成：Docker Feature](#2-groth16-证明生成docker-feature)
3. [合约验证接口：返回值类型错误](#3-合约验证接口返回值类型错误)
4. [构建错误：Metal Kernels 构建失败](#4-构建错误metal-kernels-构建失败)
5. [总结与最佳实践](#5-总结与最佳实践)

---

## 1. 依赖配置：本地路径 vs 预编译版本

### ❌ 问题描述

**现象**:
- ghostlinkV2 需要设置 `RISC0_SKIP_BUILD_KERNELS=1` 才能构建
- ghostlink (v1) 不需要任何环境变量就能正常构建

**错误信息**:
```
error: failed to run custom build command for `risc0-sys`
Could not build metal kernels
xcrun: error: unable to find utility "metal"
```

### 🔍 根本原因

**ghostlink (v1) 配置**:
```toml
# Cargo.toml
[workspace.dependencies]
risc0-zkvm = { version = "3.0.0", default-features = false }
```

**ghostlinkV2 原始配置**:
```toml
# Cargo.toml
[workspace.dependencies]
risc0-zkvm = { path = "../risc0/zkvm", default-features = false }
```

**差异分析**:
- **v1 使用 crates.io 预编译版本**: 从 crates.io 下载，已包含编译好的 Metal kernels，无需本地构建
- **v2 使用本地源码路径**: 需要从源码编译，包括 Metal kernels，但 macOS 上缺少 Xcode Command Line Tools

### ✅ 解决方案

**修改 `ghostlinkV2/Cargo.toml`**:

```toml
[workspace.dependencies]
# ❌ 错误：使用本地路径，需要构建 Metal kernels
# risc0-zkvm = { path = "../risc0/zkvm", default-features = false }

# ✅ 正确：使用 crates.io 预编译版本（与 v1 一致）
risc0-zkvm = { version = "3.0.0", default-features = false }
risc0-circuit-recursion = { version = "4.0.3", default-features = false }
risc0-circuit-keccak = { version = "4.0.3", default-features = false }
```

**效果**:
- ✅ 无需 `RISC0_SKIP_BUILD_KERNELS=1`
- ✅ 构建速度更快（无需编译 Metal kernels）
- ✅ 与 v1 保持一致的行为

### 📝 何时使用本地路径？

**使用本地路径的场景**:
- 需要修改 RISC Zero 源码
- 需要调试 RISC Zero 内部实现
- 需要测试 RISC Zero 的未发布版本

**使用预编译版本的场景（推荐）**:
- 生产环境部署
- 快速开发和测试
- 不需要修改 RISC Zero 源码

---

## 2. Groth16 证明生成：Docker Feature

### ❌ 问题描述

**现象**:
- ghostlink (v1) 能生成 Groth16 证明
- ghostlinkV2 无法生成 Groth16 证明（或需要额外配置）

**疑问**: "为什么 v1 生成的是 Groth16，v2 不是？"

### 🔍 根本原因

**ghostlink (v1) 配置**:
```toml
# host/Cargo.toml
risc0-zkvm = { version = "3.0.0", features = ["docker"] }
```

**ghostlinkV2 原始配置**:
```toml
# host/Cargo.toml
risc0-zkvm = { workspace = true, features = ["client", "std"] }
```

**差异分析**:
- **v1 启用了 `docker` feature**: 通过 Docker 容器执行 Groth16 压缩，无需本地安装 Groth16 组件
- **v2 未启用 `docker` feature**: 需要本地安装 `risc0-groth16` 组件，或使用 Dev Mode（生成假证明）

### ✅ 解决方案

**修改 `ghostlinkV2/host/Cargo.toml`**:

```toml
[dependencies]
# ❌ 错误：缺少 docker feature
# risc0-zkvm = { workspace = true, features = ["client", "std"] }

# ✅ 正确：添加 docker feature（与 v1 一致）
risc0-zkvm = { workspace = true, features = ["client", "std", "docker"] }
```

**效果**:
- ✅ 自动使用 Docker 容器执行 Groth16 压缩
- ✅ 无需本地安装 `risc0-groth16` 组件
- ✅ 生成真实的 Groth16 证明（可用于链上验证）

### 📝 Docker Feature 工作原理

1. **证明生成流程**:
   ```
   Guest 代码执行 → STARK 证明 → Docker 容器压缩 → Groth16 SNARK
   ```

2. **Docker 容器**:
   - RISC Zero 自动下载并运行 Docker 镜像
   - 镜像包含 Groth16 压缩工具
   - 无需本地安装任何额外组件

3. **前提条件**:
   - Docker 必须已安装并运行
   - 首次运行会自动下载镜像（可能需要几分钟）

---

## 3. 合约验证接口：返回值类型错误

### ❌ 问题描述

**现象**:
- 合约调用 `verifier.verify()` 时编译错误或运行时错误
- 错误信息：`TypeError: Cannot read property 'toString' of undefined`

**原始错误代码**:
```solidity
// ❌ 错误：IRiscZeroVerifier.verify() 不返回 bool
bool verified = verifier.verify(seal, imageId, journalHash);
require(verified, "ZK proof verification failed");
```

### 🔍 根本原因

**RISC Zero 标准接口**:
```solidity
interface IRiscZeroVerifier {
    // ✅ 正确：不返回值，验证失败时 revert
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external view;
}
```

**错误理解**:
- 误以为 `verify()` 返回 `bool`
- 实际上 `verify()` 不返回值，验证失败时直接 revert

### ✅ 解决方案

**修改 `ghostlinkV2/contract/IRiscZeroVerifier.sol`**:

```solidity
interface IRiscZeroVerifier {
    /**
     * @notice Verify a RISC Zero proof
     * @dev Reverts on verification failure. Returns nothing on success.
     * @param seal The proof seal (receipt) from RISC Zero (Groth16 format with selector)
     * @param imageId The Image ID of the guest program
     * @param journalHash The SHA-256 hash of the journal
     */
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external view;  // ✅ 不返回 bool
}
```

**修改 `ghostlinkV2/contract/GhostLinkSBT.sol`**:

```solidity
function mint(...) external returns (uint256 tokenId) {
    // ...

    // ❌ 错误
    // bool verified = verifier.verify(seal, imageId, journalHash);
    // require(verified, "ZK proof verification failed");

    // ✅ 正确：直接调用，失败时自动 revert
    verifier.verify(seal, imageId, journalHash);

    // 验证通过，继续执行...
}
```

**效果**:
- ✅ 符合 RISC Zero 标准接口
- ✅ 验证失败时自动 revert，无需手动检查返回值
- ✅ 代码更简洁

---

## 4. 构建错误：Metal Kernels 构建失败

### ❌ 问题描述

**现象**:
- 使用本地源码路径时，构建失败
- 错误信息：`Could not build metal kernels`

**错误信息**:
```
thread '<unnamed>' panicked at build_kernel/src/lib.rs:253:29:
Could not build metal kernels
xcrun: error: unable to find utility "metal", not a developer tool or in PATH
```

### 🔍 根本原因

**macOS 特殊要求**:
- RISC Zero 在 macOS 上需要构建 Metal kernels（GPU 加速）
- 构建 Metal kernels 需要 Xcode Command Line Tools
- 如果未安装或路径不正确，构建会失败

**触发条件**:
- 使用本地源码路径：`risc0-zkvm = { path = "../risc0/zkvm" }`
- 未安装 Xcode Command Line Tools
- 或 Xcode Command Line Tools 路径配置错误

### ✅ 解决方案

#### 方案 1: 使用预编译版本（推荐）

```toml
# ✅ 使用 crates.io 预编译版本，无需构建 Metal kernels
risc0-zkvm = { version = "3.0.0", default-features = false }
```

#### 方案 2: 安装 Xcode Command Line Tools

```bash
# 检查是否已安装
xcode-select -p

# 如果未安装，执行：
xcode-select --install

# 如果路径不对，重置：
sudo xcode-select --reset
```

#### 方案 3: 跳过内核构建（临时方案）

```bash
# 设置环境变量跳过内核构建
export RISC0_SKIP_BUILD_KERNELS=1
cargo build --release
```

**注意**: 方案 3 会影响性能，不推荐用于生产环境。

---

## 5. 总结与最佳实践

### ✅ 推荐的配置

#### Cargo.toml (Workspace)

```toml
[workspace.dependencies]
# 使用 crates.io 预编译版本
risc0-zkvm = { version = "3.0.0", default-features = false }
risc0-circuit-recursion = { version = "4.0.3", default-features = false }
risc0-circuit-keccak = { version = "4.0.3", default-features = false }
```

#### host/Cargo.toml

```toml
[dependencies]
risc0-zkvm = { workspace = true, features = ["client", "std", "docker"] }
risc0-circuit-recursion = { workspace = true }
risc0-circuit-keccak = { workspace = true }
```

#### contract/IRiscZeroVerifier.sol

```solidity
interface IRiscZeroVerifier {
    // 不返回值，验证失败时 revert
    function verify(
        bytes calldata seal,
        bytes32 imageId,
        bytes32 journalHash
    ) external view;
}
```

#### contract/GhostLinkSBT.sol

```solidity
function mint(...) external returns (uint256 tokenId) {
    // 直接调用，失败时自动 revert
    verifier.verify(seal, imageId, journalHash);

    // 验证通过，继续执行...
}
```

### 📋 检查清单

在部署前，请确认：

- [ ] 使用 crates.io 预编译版本（而非本地路径）
- [ ] 启用了 `docker` feature（用于 Groth16 压缩）
- [ ] Verifier 接口不返回 `bool`
- [ ] 合约中直接调用 `verify()`，不检查返回值
- [ ] Docker 已安装并运行（用于 Groth16 压缩）
- [ ] Journal Hash 使用 `sha256`（而非 `keccak256`）

### 🚀 启动命令

**开发模式（快速测试）**:
```bash
cd host
RISC0_DEV_MODE=1 cargo run
```

**生产模式（真实 Groth16 证明）**:
```bash
cd host
cargo run --release
```

**注意**: 使用预编译版本后，无需 `RISC0_SKIP_BUILD_KERNELS=1`。

### 🔗 相关文档

- [RISC Zero 官方文档](https://dev.risczero.com/)
- [RISC Zero Verifier 合约地址](https://dev.risczero.com/api/blockchain-integration/contracts/verifier)
- [STARTUP_GUIDE.md](STARTUP_GUIDE.md) - 启动和使用指南

---

## 📝 修改历史

| 日期 | 修改内容 | 原因 |
|------|---------|------|
| 2026-01-26 | 将依赖从本地路径改为 crates.io 预编译版本 | 避免 Metal kernels 构建错误 |
| 2026-01-26 | 添加 `docker` feature | 支持 Groth16 证明生成 |
| 2026-01-26 | 修正 Verifier 接口返回值类型 | 符合 RISC Zero 标准接口 |
| 2026-01-26 | 修正合约中的验证调用方式 | 正确处理验证失败情况 |

---

**文档版本**: v2.0
**最后更新**: 2026-01-26
