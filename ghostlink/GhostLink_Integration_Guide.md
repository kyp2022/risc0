# GhostLink 集成指南 (Integration Guide)

**版本**: v1.0.0
**网络**: Ethereum Sepolia Testnet
**更新日期**: 2024-05-22

---

## 1. 项目简介 (Overview)

GhostLink 是一个基于 RISC Zero zkVM 的去中心化数据验证协议。它允许用户生成 Web2 数据（如 GitHub 账户信息）的零知识证明，并在链上铸造 Soulbound Token (SBT) 作为声誉凭证，而无需暴露隐私数据。

### 核心流程
1.  **Off-chain**: 调用 Prover API，传入用户数据和钱包地址，生成 ZK Proof (`seal`) 和防重放标识 (`nullifier`)。
2.  **On-chain**: 调用智能合约的 `mint` 函数，提交 Proof。合约验证通过后发放 SBT。

---

## 2. 链下证明生成 API (Prover API)

该接口用于在链下生成零知识证明。

*   **Base URL**: `http://127.0.0.1:3000` (本地调试)
*   **Endpoint**: `/prove`
*   **Method**: `POST`
*   **Content-Type**: `application/json`

### 2.1 请求参数 (Request)

| 参数名 | 类型 | 必填 | 说明 |
| :--- | :--- | :--- | :--- |
| `recipient` | String | 是 | **接收者钱包地址** (0x开头)。SBT 将被绑定到此地址。 |
| `input_json` | String | 是 | **用户数据的 JSON 字符串**。注意：这是一个 String 类型的字段，内部必须是合法的 JSON 结构。 |

**`input_json` 内部结构要求：**
Guest 程序会解析这个 JSON 字符串，必须包含以下字段：
*   `id` (Number): GitHub 用户 ID (用于生成唯一 Nullifier)。
*   `login` (String): 用户名。
*   `created_at` (String): 注册时间 (ISO 8601格式, e.g., "2020-01-01T...")。
*   `public_repos` (Number): 公开仓库数量。

**业务验证规则 (Guest Logic):**
*   `public_repos` 必须大于 0。
*   `created_at` 年份必须早于 2023 年。

### 2.2 请求示例 (Example)

```bash
curl -X POST http://127.0.0.1:3000/prove \
     -H "Content-Type: application/json" \
     -d '{
           "recipient": "0x534C283D6339183d20c2e7f0fd6522d9e6CD5145",
           "input_json": "{\"id\": 12345, \"login\": \"partner_user\", \"created_at\": \"2022-05-20T00:00:00Z\", \"public_repos\": 10}"
         }'
```

### 2.3 响应参数 (Response)

**成功 (HTTP 200):**

```json
{
  "status": "success",
  "receipt_hex": "0x...",      // [核心] ZK Proof 数据 (Seal)，用于合约 verify
  "nullifier_hex": "...",      // [核心] 防重放哈希，用于合约 mint
  "image_id_hex": "...",       // 当前 Guest 程序的 ImageID (用于核对合约版本)
  "journal_hex": "..."         // 原始日志数据 (调试用)
}
```

**失败:**

```json
{
  "status": "Proving failed: Account is too new! Must be created before 2023.",
  "receipt_hex": "",
  ...
}
```

---

## 3. 链上合约交互 (Smart Contract)

拿到 API 返回的数据后，前端或脚本需调用合约进行铸造。

### 3.1 合约信息 (Sepolia)

*   **GhostLinkSBT 地址**: `0x624365303EEacb9E098DdB892244D616a7d44510` (请以最新部署为准)
*   **RiscZeroVerifierRouter**: `0x925d8331ddc0a1F0d96E68CF073DFE1d92b69187` (Sepolia Official)

### 3.2 Mint 函数签名

```solidity
function mint(
    bytes calldata seal,       // 对应 API 返回的 receipt_hex (需加 0x)
    address recipient,         // 对应请求时的 recipient 地址
    bytes32 nullifier          // 对应 API 返回的 nullifier_hex (需加 0x)
) external;
```

### 3.3 调用示例 (Ethers.js)

```javascript
const contract = new ethers.Contract(CONTRACT_ADDRESS, ABI, signer);

// 1. 准备参数
// 注意：API 返回的 hex 字符串如果没有 0x 前缀，需要手动加上
const seal = "0x" + apiResponse.receipt_hex;
const nullifier = "0x" + apiResponse.nullifier_hex;
const recipient = userWalletAddress;

// 2. 发起交易
// 建议手动设置 gasLimit，防止因 Revert 导致的估算错误
try {
    const tx = await contract.mint(seal, recipient, nullifier, {
        gasLimit: 500000
    });
    console.log("Transaction sent:", tx.hash);
    await tx.wait();
    console.log("Mint success!");
} catch (error) {
    console.error("Mint failed:", error);
}
```

---

## 4. 常见问题排查 (Troubleshooting)

| 错误现象 | 可能原因 | 解决方案 |
| :--- | :--- | :--- |
| **Gas Limit too high / Transaction Reverted** | 1. ImageID 不匹配<br>2. Verifier 地址错误<br>3. Proof 无效 | 1. 检查 API 返回的 `image_id_hex` 是否与合约部署时的一致。<br>2. 确保合约部署时使用了正确的 Sepolia Verifier 地址。<br>3. 强制设置 `gasLimit: 500000` 查看链上具体报错。 |
| **Proving failed** | 用户数据不满足条件 | 检查 `input_json` 中的 `created_at` 是否早于 2023，且 `public_repos > 0`。 |
| **Invalid bytes parameter** | ABI 编码不匹配 | 确保使用最新的合约版本，该版本已改为显式接收 `(seal, recipient, nullifier)` 参数。 |
