# GhostLink V2 快速参考

> 快速查找常用命令和参数

---

## 🚀 快速启动

### 启动后端服务

```bash
# 开发模式（快速测试）
cd host
RISC0_DEV_MODE=1 cargo run

# 生产模式（真实证明）
cd host
cargo run --release --features prove
```

**服务地址**: `http://localhost:3000`

---

## 📝 API 调用

### 生成证明

```bash
curl -X POST http://localhost:3000/api/v1/prove \
  -H "Content-Type: application/json" \
  -d '{
    "credential_type": "github",
    "data": { ... },
    "recipient": "0x..."
  }'
```

### 凭证类型映射

| 类型 | 值 | 数据字段 |
|------|-----|---------|
| GitHub | `"github"` | `user_id`, `username`, `created_at`, `public_repos` |
| Alipay | `"alipay"` | `balance`, `id_number_hash`, `threshold` |
| Twitter | `"twitter"` | `user_id`, `handle`, `created_at` |
| Wallet | `"wallet"` | `address`, `balance_wei`, `transaction_count`, `chain_id`, `signature`, `message` |

---

## 🔗 合约调用

### Mint 函数签名

```solidity
function mint(
    bytes calldata seal,      // receipt_hex (带 0x)
    bytes32 nullifier,        // nullifier_hex (带 0x)
    CredentialType credType   // 0=GitHub, 1=Alipay, 2=Twitter, 3=Wallet
) external returns (uint256)
```

### 使用 Cast 调用

```bash
cast send $CONTRACT_ADDRESS \
  "mint(bytes,bytes32,uint8)" \
  $SEAL \
  $NULLIFIER \
  $CRED_TYPE \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --gas-limit 500000
```

### 参数来源

| 参数 | 来源 | 格式 |
|------|------|------|
| `seal` | API 响应的 `receipt_hex` | `0x` + hex字符串 |
| `nullifier` | API 响应的 `nullifier_hex` | `0x` + hex字符串 |
| `credType` | 凭证类型 | `0-3` (uint8) |

---

## 🔑 重要地址

### RISC Zero Verifier

- **Sepolia**: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187`
- **Mainnet**: `0x8EaB2D97Dfce405A1692a21b3ff3A172d593D319`

---

## 📊 数据格式

### Journal 格式

```
[20字节address][32字节nullifier][1字节type] = 53字节
```

### Seal 格式

```
[4字节选择器][192字节Groth16证明] = 196字节
```

### Nullifier 生成规则

- **GitHub**: `keccak256("github" || user_id)`
- **Alipay**: `keccak256("alipay" || id_number_hash)`
- **Twitter**: `keccak256("twitter" || user_id)`
- **Wallet**: `keccak256("wallet" || address || chain_id)`

---

## ⚠️ 常见问题

### Image ID 不匹配

```bash
# 检查合约中的 Image ID
cast call $CONTRACT_ADDRESS "imageId()" --rpc-url $RPC_URL

# 对比后端服务启动日志中的 Image ID
```

### Nullifier 已使用

```bash
# 检查 nullifier 状态
cast call $CONTRACT_ADDRESS "nullifiers(bytes32)" $NULLIFIER --rpc-url $RPC_URL
```

### 证明格式错误

- ✅ 确保使用生产模式（禁用 `RISC0_DEV_MODE`）
- ✅ 确保安装了 `risc0-groth16` 组件
- ✅ 确保 `seal` 包含 `0x` 前缀

---

## 📚 完整文档

详细说明请参考：[STARTUP_GUIDE.md](STARTUP_GUIDE.md)
