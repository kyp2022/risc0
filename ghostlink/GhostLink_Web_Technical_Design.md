# GhostLink Web 架构设计文档 (Web-First MVP)

## 1. 系统架构图 (System Architecture)

采用 **"Thick Client, Thin Server"** (重客户端，轻服务端) 架构。

```mermaid
graph TD
    subgraph "User's Browser (Client)"
        UI[React / Next.js Frontend]
        DataFetcher[Data Fetcher (API Proxy)]

        subgraph "WASM Worker (Local Privacy Zone)"
            ZK_Prover[RISC Zero Prover (WASM)]
            Guest_Code[Guest Logic (Rust)]
        end

        UI -->|1. Login & Fetch| DataFetcher
        DataFetcher -->|2. Raw JSON| ZK_Prover
        ZK_Prover -->|3. Generate Proof| UI
    end

    subgraph "GhostLink Cloud (Minimal Backend)"
        Relayer[Proof Relayer Service]
        Registry[Credential Registry]
    end

    subgraph "Blockchain (Ethereum/L2)"
        Verifier[Verifier Contract]
        SBT[Soulbound Token Contract]
    end

    UI -->|4. Submit Proof| Relayer
    Relayer -->|5. Gasless Transaction| Verifier
    Verifier -->|6. Mint SBT| SBT
```

---

## 2. 核心模块设计

### 2.1 前端 (Frontend)
*   **技术栈**: Next.js (React), TailwindCSS, Viem (以太坊交互).
*   **核心功能**:
    *   **OAuth 集成**: 处理 GitHub/Twitter 登录回调。
    *   **WASM 加载器**: 使用 Web Worker 异步加载 `risc0-prover.wasm`，避免阻塞 UI 线程。
    *   **进度条**: 因为本地生成证明较慢，需要设计友好的进度提示（"正在加密您的数据..."）。

### 2.2 零知识核心 (ZK Core - WASM)
这是运行在浏览器里的 Rust 代码。
*   **技术栈**: Rust, `wasm-bindgen`, `risc0-zkvm`.
*   **编译目标**: `wasm32-unknown-unknown`.
*   **职责**:
    *   接收前端传来的 JSON 字符串。
    *   运行 Guest 代码进行逻辑验证（如 `followers > 100`）。
    *   生成 Receipt（证明）。

### 2.3 后台管理系统 (Backend)
*   **技术栈**: Node.js / Go, PostgreSQL (仅存配置).
*   **功能**:
    *   **Relayer**: 接收用户的 Proof，检查是否有效，然后用项目方的钱包发起链上交易（为用户支付 Gas，降低用户门槛）。
    *   **配置中心**: 动态下发最新的 `ImageID`（当 Guest 代码更新时，前端无需重新发版，只需从后台获取新的 ID）。

---

## 3. 数据流与隐私设计 (Data Flow & Privacy)

### 场景：生成 GitHub 开发者凭证

1.  **数据获取**:
    *   用户在前端点击 "Connect GitHub"。
    *   前端通过 OAuth 获取 Access Token。
    *   前端直接调用 `api.github.com/user` 获取 JSON 数据。
    *   *注意*: 为了防止用户伪造 API 返回结果，进阶版需引入 **TLSNotary** (通过代理服务器对 HTTPS 流量进行公证，但不解密内容)。MVP 阶段可暂略，假设 API 返回即真实。

2.  **本地证明 (The "Magic" Box)**:
    *   前端将 JSON 数据传入 WASM Worker。
    *   **关键点**: 这一步完全在内存中进行，不涉及网络传输。
    *   WASM 运行 Rust 逻辑，验证数据，生成证明。
    *   WASM 返回 `Receipt` (二进制) 和 `Journal` (公开输出：如 "GitHub ID Hash: 0x123...")。

3.  **上链**:
    *   前端将 `Receipt` 发送给后台 Relayer。
    *   后台提交到链上 Verifier 合约。
    *   合约验证通过，给用户地址 Mint 一个 "Dev-Pass" SBT。

---

## 4. 凭证存储策略 (Credential Storage)

### 4.1 结果凭证 (The Badge) - **On-Chain**
*   **形式**: Soulbound Token (SBT) / NFT。
*   **存储位置**: 区块链 (用户钱包地址)。
*   **特性**: 公开、永久、不可转移。这是用户在第三方应用中展示身份的唯一凭证。

### 4.2 原始证明 (The Receipt) - **Ephemeral (临时)**
*   **形式**: 二进制 ZK Proof 文件。
*   **存储位置**: 浏览器内存 (RAM)。
*   **生命周期**: 生成 -> 提交上链 -> 验证成功 -> **销毁**。
*   **原因**: 一旦链上 SBT 铸造成功，原始证明就不再需要了。为了保护隐私和减少用户负担，不建议持久化存储原始证明文件。

---

## 5. 开发路线图 (Web MVP)

### Phase 1: 核心验证 (Core Logic)
*   **目标**: 在浏览器控制台跑通 "Input JSON -> Output Proof"。
*   **任务**:
    1.  编写 Rust Guest 代码 (解析 JSON)。
    2.  配置 `risc0` 的 WASM 构建流程。
    3.  编写简单的 HTML 页面调用 WASM。

### Phase 2: 前端与合约 (App & Chain)
*   **目标**: 完整的用户界面和 NFT 发放。
*   **任务**:
    1.  搭建 Next.js 项目。
    2.  部署 Solidity Verifier 合约。
    3.  实现前端与合约的交互。

### Phase 3: 后台与优化 (Backend & Polish)
*   **目标**: 提升体验，免 Gas。
*   **任务**:
    1.  搭建 Relayer 服务。
    2.  优化 WASM 体积和加载速度。
