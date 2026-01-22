# GhostLink: 去中心化零知识数据护照 (Product Specification)

## 1. 产品愿景 (Vision)
**GhostLink** 旨在成为 Web3 世界的“数据护照”基础设施。它利用 RISC Zero 的通用计算能力和零知识证明技术，允许用户将 Web2 的高价值行为数据（如 GitHub 贡献、Twitter 影响力、银行流水）以**完全隐私**的方式导入 Web3，生成链上可验证的凭证（SBT/NFT），从而解决 Web3 的身份信任缺失问题，同时保护用户隐私。

**Slogan**: *Your Reputation, Unchained & Unseen.*

---

## 2. 核心痛点 (Problem Statement)

### 2.1 Web3 项目方
*   **女巫攻击 (Sybil Attacks)**: 空投和 DAO 治理常被脚本账号刷爆，无法识别真实用户。
*   **缺乏用户画像**: 链上地址是匿名的，无法区分高净值用户、开发者或普通散户，导致营销效率低下。
*   **信用贷缺失**: 由于无法评估借款人的链下信用，DeFi 只能进行低效的超额抵押。

### 2.2 Web3 用户
*   **隐私顾虑**: 用户拒绝直接上传身份证或银行流水来证明身份。
*   **声誉割裂**: 用户在 Web2 积累的十年信誉（如 GitHub 提交记录、Steam 游戏时长）在 Web3 毫无价值。

---

## 3. 解决方案 (Solution: GhostLink)

GhostLink 是一个基于 **RISC Zero zkVM** 和 **zkTLS (TLSNotary)** 技术的隐私数据桥接协议。

### 3.1 核心工作流
1.  **数据获取 (Data Fetching)**: 用户在本地客户端（GhostLink App/Plugin）登录 Web2 平台（如 GitHub）。客户端通过 zkTLS 技术拦截并验证 HTTPS 响应，确保数据真实来自服务器且未被篡改。
2.  **隐私计算 (Private Computation)**: 原始数据（如 JSON/HTML）被送入本地运行的 **RISC Zero zkVM**。
3.  **逻辑验证 (Logic Verification)**: zkVM 执行 Rust 编写的 Guest 程序，解析数据并验证特定条件（例如：`followers > 1000` 或 `account_age > 3 years`）。
4.  **证明生成 (Proof Generation)**: zkVM 生成一个零知识证明 (Receipt)，证明用户满足条件，并输出一个脱敏的 Nullifier（防止双花）。
5.  **链上验证 (On-Chain Verification)**: 用户将证明提交到链上 Verifier 合约，合约验证通过后铸造 SBT 或发放奖励。

---

## 4. 技术架构 (Technical Architecture)

### 4.1 客户端 (Client / Prover)
*   **技术栈**: Rust, WebAssembly (Wasm), TLSNotary
*   **功能**:
    *   管理 TLS 会话，获取签名数据。
    *   运行 RISC Zero Prover，生成 zk-SNARK 证明。
    *   **关键优势**: 利用 RISC Zero 对 Rust 的原生支持，直接复用 `serde_json`, `regex`, `html_parser` 等库处理复杂的 Web2 数据。

### 4.2 链下验证层 (zkVM Guest Code)
*   **语言**: Rust
*   **核心逻辑**:
    *   **输入**: HTTPS 响应体 (Ciphertext/Plaintext), TLS 签名。
    *   **处理**:
        1.  验证 TLS 签名（确保数据源可信）。
        2.  解析 HTTP Body (JSON/HTML)。
        3.  执行业务断言 (e.g., `assert!(user.contribution_count > 50)`).
        4.  生成 Nullifier: `Hash(user_id + salt)`，确保匿名性。
    *   **输出**: 公开输出 (Journal) 包含 `(Requirement_Met, Nullifier, Timestamp)`。

### 4.3 链上验证层 (Verifier Contract)
*   **语言**: Solidity
*   **功能**:
    *   接收 zk-Proof 和 Journal。
    *   调用 `RISCZeroVerifier` 合约验证证明有效性。
    *   检查 Nullifier 是否已使用。
    *   触发回调 (Mint NFT / Grant Access)。

---

## 5. 初始产品功能 (MVP Features)

### 5.1 "Dev-Pass" (开发者护照)
*   **数据源**: GitHub API
*   **验证逻辑**:
    *   账号注册时间 > 1 年。
    *   在特定仓库（如 `risc0/risc0`）有过 PR 合并记录。
*   **用途**: 开发者社区准入、黑客松报名资格。

### 5.2 "Social-Pass" (社交达人证)
*   **数据源**: Twitter/X API (或 HTML 归档)
*   **验证逻辑**:
    *   粉丝数 > 500。
    *   推文数量 > 100。
*   **用途**: 防止空投女巫攻击。

### 5.3 "Asset-Pass" (资产证明)
*   **数据源**: 银行/交易所 API
*   **验证逻辑**:
    *   持有资产 > $10,000 (不透露具体金额)。
*   **用途**: RWA 平台白名单。

---

## 6. 竞争优势 (Competitive Advantage)

1.  **开发速度快**: 相比 Circom/Halo2 需要重写解析逻辑，GhostLink 利用 RISC Zero 直接运行 Rust 解析器，适配 Web2 API 变动只需几分钟。
2.  **处理非结构化数据**: 能够轻松处理 Email、HTML、PDF 等复杂格式，这是传统 ZK 电路的噩梦。
3.  **真·去中心化**: 所有证明生成均在用户端完成，私钥和原始数据从未离开用户设备。

---

## 7. 路线图 (Roadmap)

*   **Phase 1 (Q1)**: 完成 "Dev-Pass" MVP，打通 GitHub -> zkVM -> Ethereum 流程。
*   **Phase 2 (Q2)**: 集成 TLSNotary，实现通用的 HTTPS 数据验证框架。
*   **Phase 3 (Q3)**: 发布 SDK，允许第三方 dApp 定义自己的验证逻辑（"Bring Your Own Logic"）。
*   **Phase 4 (Q4)**: 推出 GhostLink 移动端 App，支持 NFC 读取护照芯片（ZK-Passport）。
