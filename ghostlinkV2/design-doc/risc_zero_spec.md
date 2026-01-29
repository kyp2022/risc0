# GhostLink × RISC Zero 对接规范

> **文档版本**: v3.0  
> **日期**: 2026-01-26  
> **状态**: 待评审  
> **双方确认后生效**

---

## 1. 项目概述

**GhostLink** 是一个隐私保护的身份认证平台，用户可以将 Web2/Web3 身份凭证转化为链上可验证的 SBT（灵魂绑定代币）。

**零知识证明目标**：证明"用户满足某条件"而不暴露原始数据。

---

## 2. 凭证类型概览

| 凭证类型 | 数据来源 | 采集方 | 证明目标 |
|----------|----------|--------|----------|
| GitHub | GitHub OAuth API | GhostLink 后端 | 证明拥有 GitHub 账户 |
| 支付宝 | 资产证明 PDF | GhostLink 后端 | 证明资产 ≥ 阈值 |
| Twitter | Twitter OAuth API | GhostLink 后端 | 证明拥有 Twitter 账户 |
| 钱包资产 | 区块链 RPC | GhostLink 前端 | 证明链上资产情况 |

---

## 3. 数据流架构

```
┌──────────────────────────────────────────────────────────────┐
│                     GhostLink 系统                            │
├────────────────┬─────────────────┬───────────────────────────┤
│    前端        │     后端        │                           │
│    (React)     │     (Java)      │                           │
├────────────────┼─────────────────┤                           │
│ 钱包资产采集 ──┼─→ API 转发 ─────┼──→ RISC Zero 服务         │
│                │ GitHub OAuth ───┼──→ (由贵方部署)            │
│                │ 支付宝PDF解析 ──┼──→                         │
│                │ Twitter OAuth ──┼──→                         │
└────────────────┴─────────────────┴───────────────────────────┘
                                            │
                                            ▼
                                   ┌─────────────────┐
                                   │  智能合约验证    │
                                   │  铸造 SBT       │
                                   └─────────────────┘
```

---

## 4. 接口规范

### 4.1 统一请求格式

```http
POST /api/v1/prove
Content-Type: application/json
Authorization: Bearer {api_key}

{
  "credential_type": "github | alipay | twitter | wallet",
  "data": { ... },
  "recipient": "0x..."
}
```

### 4.2 统一响应格式

**成功**：
```json
{
  "status": "success",
  "receipt_hex": "...",
  "journal_hex": "...",
  "image_id_hex": "...",
  "nullifier_hex": "..."
}
```

**失败**：
```json
{
  "status": "error",
  "error_code": "INVALID_DATA | THRESHOLD_NOT_MET | ...",
  "message": "描述信息"
}
```

---

## 5. 凭证类型详细规范

### 5.1 GitHub 凭证

**请求**：
```json
{
  "credential_type": "github",
  "data": {
    "user_id": 12345678,
    "username": "ghostlink-user",
    "created_at": "2020-01-01T00:00:00Z",
    "public_repos": 5
  },
  "recipient": "0x..."
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `user_id` | Number | ✅ | GitHub 用户 ID |
| `username` | String | ✅ | GitHub 用户名 |
| `created_at` | String | ✅ | 账户创建时间（ISO 8601） |
| `public_repos` | Number | ✅ | 公开仓库数量 |

**ZK 验证逻辑**：
1. 验证数据格式有效
2. 生成 `nullifier = keccak256("github" || user_id)`

---

### 5.2 支付宝资产凭证

**请求**：
```json
{
  "credential_type": "alipay",
  "data": {
    "balance": "15975.01",
    "id_number_hash": "0x7a8b9c...",
    "threshold": "10000"
  },
  "recipient": "0x..."
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `balance` | String | ✅ | 总资产金额（元） |
| `id_number_hash` | String | ✅ | `keccak256(身份证号)` |
| `threshold` | String | ✅ | 资产门槛（元） |

**ZK 验证逻辑**：
1. 验证 `balance >= threshold`
2. 生成 `nullifier = keccak256("alipay" || id_number_hash)`

> **注意**：身份证号由 GhostLink 后端哈希后传入，ZK 服务不接收原文。

---

### 5.3 Twitter 凭证

**请求**：
```json
{
  "credential_type": "twitter",
  "data": {
    "user_id": "987654321",
    "handle": "ghostlink",
    "created_at": "2015-01-01T00:00:00Z",
    "followers_count": 100
  },
  "recipient": "0x..."
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `user_id` | String | ✅ | Twitter 用户 ID |
| `handle` | String | ✅ | @ 用户名（不含 @） |
| `created_at` | String | ✅ | 账户创建时间 |
| `followers_count` | Number | ❌ | 粉丝数（可选） |

**ZK 验证逻辑**：
1. 验证数据格式有效
2. 生成 `nullifier = keccak256("twitter" || user_id)`

---

### 5.4 钱包资产凭证 ⭐ 新增

**请求**：
```json
{
  "credential_type": "wallet",
  "data": {
    "address": "0x1234...abcd",
    "balance_wei": "1000000000000000000",
    "transaction_count": 42,
    "chain_id": 11155111,
    "signature": "0xabc123...",
    "message": "GhostLink Asset-Pass Verification..."
  },
  "recipient": "0x..."
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `address` | String | ✅ | 钱包地址 |
| `balance_wei` | String | ✅ | ETH 余额（Wei 单位） |
| `transaction_count` | Number | ✅ | 链上交易数量 |
| `chain_id` | Number | ✅ | 链 ID（11155111 = Sepolia） |
| `signature` | String | ✅ | 用户签名 |
| `message` | String | ✅ | 被签名的消息原文 |

**ZK 验证逻辑**：
1. **验证签名**：`ecrecover(message, signature) == address`
2. 验证 `transaction_count >= 10`（可选门槛）
3. 生成 `nullifier = keccak256("wallet" || address || chain_id)`

> **重要**：钱包数据由前端采集，需要在 ZK 电路中验证签名以防伪造。

---

## 6. Nullifier 规范

| 凭证类型 | Nullifier 计算公式 |
|----------|---------------------|
| GitHub | `keccak256("github" \|\| user_id)` |
| Alipay | `keccak256("alipay" \|\| id_number_hash)` |
| Twitter | `keccak256("twitter" \|\| user_id)` |
| Wallet | `keccak256("wallet" \|\| address \|\| chain_id)` |

> 同一用户使用同一身份只能铸造一次 SBT。

---

## 7. 部署与对接

### 7.1 服务端点

贵方需提供以下环境的服务地址：

| 环境 | 用途 | URL（待填写） |
|------|------|---------------|
| 开发 | 本地联调 | `http://localhost:3000` |
| 测试 | Sepolia 测试 | `https://zk-test.xxx.com` |
| 生产 | 主网上线 | `https://zk.xxx.com` |

### 7.2 性能要求

| 指标 | 开发环境 | 生产环境 |
|------|----------|----------|
| 证明生成时间 | ≤ 10 分钟 | ≤ 60 秒 |
| 并发支持 | 1 | ≥ 20 |
| 可用性 | - | ≥ 99.9% |

### 7.3 安全要求

- 所有接口使用 HTTPS（生产环境）
- 支持 API Key 认证
- 不记录 `id_number_hash` 等敏感字段
- 证明数据不可伪造

---

## 8. 交付物

贵方需交付：

| 交付物 | 说明 |
|--------|------|
| HTTP API 服务 | 按上述规范实现 |
| Image ID | 各凭证类型的 Guest 程序 ID |
| Verifier 合约地址 | 部署到 Sepolia 的验证器地址 |
| API 文档 | 实际接口详细说明 |
| 测试用例 | 示例请求和预期响应 |

---

