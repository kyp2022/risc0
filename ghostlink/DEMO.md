# GhostLink 功能清单

## ✅ 已实现的功能

### 1. 钱包连接
- ✅ 连接 MetaMask 钱包
- ✅ 显示连接的钱包地址
- ✅ 地址格式验证和标准化

### 2. 合约部署
- ✅ 选择 Verifier 地址（Router 或 Groth16）
- ✅ 部署 GhostLinkSBT 合约
- ✅ 自动填充 Image ID
- ✅ 显示部署地址

### 3. ZK 证明生成
- ✅ 输入 GitHub 用户 JSON 数据
- ✅ 生成 Groth16 证明
- ✅ 显示 Image ID、Journal、Nullifier 等信息
- ✅ JSON 格式验证
- ✅ 示例数据加载

### 4. SBT Mint（铸造）
- ✅ 使用 ZK 证明 mint SBT
- ✅ Image ID 匹配验证
- ✅ Nullifier 重复使用检查
- ✅ 参数格式验证
- ✅ Gas 估算和交易发送
- ✅ 交易状态显示

### 5. 验证和查询功能 ✨ **新增**

#### 5.1 检查我的 SBTs
- ✅ 查看当前钱包拥有的所有 SBT
- ✅ 显示 Token ID、Owner、Token URI
- ✅ 以表格形式展示

#### 5.2 验证地址
- ✅ 检查指定地址是否拥有 GhostLink SBT
- ✅ 显示拥有的 SBT 数量
- ✅ 地址格式验证

#### 5.3 检查 Nullifier
- ✅ 检查 nullifier 是否已被使用
- ✅ Nullifier 格式验证
- ✅ 显示使用状态

#### 5.4 查询 Token 信息
- ✅ 通过 Token ID 查询 token 信息
- ✅ 显示 Owner、Token URI
- ✅ 检查是否为当前用户的 token

#### 5.5 合约信息
- ✅ 查看合约基本信息
- ✅ 显示合约地址、名称、符号
- ✅ 显示 Image ID、Verifier 地址
- ✅ 显示已 mint 的 token 总数

## 📋 功能使用说明

### 检查我的 SBTs

1. 确保已连接钱包
2. 输入合约地址
3. 点击 "Check My SBTs"
4. 查看拥有的所有 SBT 列表

**显示信息**：
- Token ID
- Owner 地址
- Token URI（如果有）

### 验证地址

1. 输入要验证的地址（0x...）
2. 点击 "Verify Address"
3. 查看验证结果

**结果**：
- ✅ 如果拥有 SBT：显示拥有的数量
- ❌ 如果不拥有：显示未验证

### 检查 Nullifier

1. 输入 nullifier（0x + 64 hex chars）
2. 点击 "Check Nullifier"
3. 查看使用状态

**结果**：
- ⚠️ 已使用：无法再次使用
- ✅ 未使用：可以用于 mint

### 查询 Token 信息

1. 输入 Token ID（数字）
2. 点击 "Query Token"
3. 查看 token 详细信息

**显示信息**：
- Token ID
- Owner 地址
- Token URI
- 是否为当前用户的 token

### 合约信息

1. 输入合约地址
2. 点击 "Get Contract Info"
3. 查看合约所有信息

**显示信息**：
- 合约地址
- 名称（GhostLink SBT）
- 符号（GLSBT）
- Image ID
- Verifier 地址
- 已 mint 的 token 总数

## 🔧 技术实现

### 使用的合约函数

```solidity
// ERC721 标准函数
balanceOf(address owner) → uint256
ownerOf(uint256 tokenId) → address
tokenURI(uint256 tokenId) → string

// GhostLinkSBT 特定函数
nullifiers(bytes32) → bool
imageId() → bytes32
verifier() → address
name() → string
symbol() → string
```

### 查询优化

- **Token 遍历**：通过 `ownerOf` 遍历查找所有 token（最多检查 1000 个）
- **错误处理**：优雅处理不存在的 token
- **状态管理**：按钮根据合约地址和钱包连接状态自动启用/禁用

## 🎯 使用场景

### 场景 1: 验证用户身份
使用 "验证地址" 功能检查用户是否拥有 GhostLink SBT，证明其 GitHub 身份已验证。

### 场景 2: 防止重复 mint
使用 "检查 Nullifier" 功能在 mint 前验证 nullifier 是否已被使用。

### 场景 3: 查看我的凭证
使用 "检查我的 SBTs" 功能查看所有已获得的身份凭证。

### 场景 4: 查询特定凭证
使用 "查询 Token 信息" 功能查看特定 token 的详细信息。

### 场景 5: 合约审计
使用 "合约信息" 功能查看合约的配置和状态。

## 📝 注意事项

1. **Token 遍历限制**：当前最多检查 1000 个 tokenId，如果 token 数量超过此限制，可能需要优化查询方式
2. **Gas 成本**：查询操作是只读的，不需要 gas，但 mint 操作需要 gas
3. **网络要求**：确保连接到正确的网络（Sepolia 或 Mainnet）
4. **合约地址**：确保输入的合约地址正确

## 🚀 未来可能的改进

- [ ] 添加事件监听，实时更新 SBT 列表
- [ ] 支持批量查询多个地址
- [ ] 添加 Token URI 的元数据显示
- [ ] 支持导出 SBT 列表为 JSON
- [ ] 添加搜索功能（按地址、tokenId 搜索）
- [ ] 优化大量 token 的查询性能（使用事件日志）

