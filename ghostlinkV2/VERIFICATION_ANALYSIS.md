# GhostLink V2 验证模块分析报告

## ✅ Groth16 格式检查

### Host 代码（证明生成）

**位置**: `host/src/main.rs` 第 241-254 行

**检查结果**: ✅ **格式正确**

```rust
// 1. 压缩为 Groth16
let compress_result = prover.compress(&ProverOpts::groth16(), &receipt);

// 2. 提取 Groth16 证明数据
if let Ok(groth16_inner) = groth16_receipt.inner.groth16() {
    let verifier_params = groth16_inner.verifier_parameters;
    let seal_data = groth16_inner.seal.clone();

    // 3. 构建包含选择器的 seal
    let selector = &verifier_params.as_bytes()[..4];
    let mut seal_bytes = Vec::with_capacity(selector.len() + seal_data.len());
    seal_bytes.extend_from_slice(selector);      // 4 字节选择器
    seal_bytes.extend_from_slice(seal_data.as_ref());  // Groth16 证明数据
}
```

**Seal 格式**:
```
[4字节选择器][Groth16证明数据]
├─ Selector: verifier_parameters 的前4字节（用于路由）
└─ Groth16 Proof: ABI编码的 (A, B, C) 椭圆曲线点
   ├─ A: uint256[2] (G1点)
   ├─ B: uint256[2][2] (G2点)
   └─ C: uint256[2] (G1点)
```

**总大小**: 约 196 字节（4字节选择器 + 192字节Groth16证明）

---

## ✅ 合约验证支持检查

### 问题 1: Verifier 接口定义 ❌ → ✅ 已修复

**原问题**: `IRiscZeroVerifier.verify()` 返回 `bool`，但标准接口不返回值，验证失败时 revert。

**修复前**:
```solidity
function verify(...) external view returns (bool success);
```

**修复后**:
```solidity
function verify(...) external view;  // Reverts on failure
```

### 问题 2: 合约中的验证调用 ❌ → ✅ 已修复

**修复前**:
```solidity
bool verified = verifier.verify(seal, imageId, journalHash);
require(verified, "ZK proof verification failed");
```

**修复后**:
```solidity
// verifier.verify() 验证失败时会 revert，成功时无返回值
verifier.verify(seal, imageId, journalHash);
```

---

## ✅ Journal 格式匹配检查

### Guest 代码输出

**位置**: `methods/guest/src/main.rs` 第 241-258 行

```rust
// Journal 格式: [20 bytes address][32 bytes nullifier][1 byte type] = 53 bytes
let mut journal = Vec::with_capacity(53);
journal.extend_from_slice(&input.recipient);  // 20 bytes
journal.extend_from_slice(&nullifier);        // 32 bytes
journal.push(credType as u8);                 // 1 byte
env::commit_slice(&journal);
```

### 合约中的 Journal Hash 计算

**位置**: `contract/GhostLinkSBT.sol` 第 113-115 行

```solidity
bytes32 journalHash = sha256(
    abi.encodePacked(msg.sender, nullifier, uint8(credType))
);
```

**匹配性**: ✅ **完全匹配**

- `abi.encodePacked()` 会将参数紧密打包：
  - `msg.sender` (address, 20 bytes)
  - `nullifier` (bytes32, 32 bytes)
  - `uint8(credType)` (1 byte)
- 总计: 53 字节，与 Guest 代码输出的 journal 完全一致
- `sha256()` 计算后得到 32 字节的 `journalHash`

---

## 📋 验证流程总结

```
1. Host 生成证明
   ├─ 执行 Guest 代码
   ├─ 生成 STARK 证明
   └─ 压缩为 Groth16 SNARK
      └─ Seal = [4字节选择器][192字节Groth16证明]

2. Guest 输出 Journal
   └─ [20字节address][32字节nullifier][1字节type] = 53字节

3. 合约验证
   ├─ 计算 journalHash = sha256(abi.encodePacked(msg.sender, nullifier, credType))
   ├─ 调用 verifier.verify(seal, imageId, journalHash)
   └─ 验证失败时 revert，成功时继续执行
```

---

## ⚠️ 注意事项

### 1. Dev Mode 警告

当前代码在 `RISC0_DEV_MODE` 环境下会生成假的证明：
- 返回 `"00000000_fake_receipt_dev_mode"` 作为 seal
- **生产环境必须禁用 Dev Mode**

### 2. Groth16 压缩要求

- 需要安装 `risc0-groth16` 组件或启用 `docker` feature
- 可能需要 GPU 支持（CUDA）或 Docker 环境
- 压缩时间：几分钟（取决于硬件）

### 3. Verifier 地址

确保使用正确的 RISC Zero Verifier Router 地址：
- **Sepolia**: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`
- **Mainnet**: `0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`

### 4. ⚠️ 关键：Journal Hash 必须使用 SHA-256

**重要**: RISC Zero 的链上验证器 (`IRiscZeroVerifier`) 的核心逻辑是基于 **SHA-256** 的。

**原因**:
- RISC Zero 源码中 Journal 的 digest 实现使用 SHA-256：`risc0/zkvm/src/receipt.rs`
- 合约验证器内部使用 SHA-256 计算 journal digest
- 如果合约使用 `keccak256()`，哈希值不匹配，验证会失败

**正确做法**:
```solidity
// ✅ 正确：使用 sha256
bytes32 journalHash = sha256(
    abi.encodePacked(msg.sender, nullifier, uint8(credType))
);

// ❌ 错误：不要使用 keccak256
// bytes32 journalHash = keccak256(...);  // 会导致验证失败
```

**注意**: 
- Nullifier 的生成可以使用 `keccak256`（这是业务逻辑）
- 但 Journal Hash 的计算必须使用 `sha256`（这是 RISC Zero 的要求）

---

## ✅ 最终结论

1. **Groth16 格式**: ✅ 正确
   - Host 代码正确使用 `ProverOpts::groth16()`
   - Seal 格式正确：`[选择器][Groth16证明]`

2. **合约验证支持**: ✅ 已修复
   - Verifier 接口已修正
   - 合约调用方式已修正

3. **Journal 格式匹配**: ✅ 完全匹配
   - Guest 输出 53 字节
   - 合约计算 `sha256(abi.encodePacked(...))` 匹配

**建议**: 
- 在非 Dev Mode 环境下测试完整的证明生成和验证流程
- 确保 Groth16 压缩功能正常工作
- 在测试网上部署合约并验证端到端流程
