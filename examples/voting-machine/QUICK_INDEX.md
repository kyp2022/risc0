# 投票机示例 - 快速索引与纲要

> 这是一份快速参考指南。详细内容请查看相应的文档。

## 📚 文档导航

| 文档 | 用途 | 阅读时间 | 适合人群 |
|------|------|---------|---------|
| **LEARNING_GUIDE.md** | 完整讲解 + Rust 基础 | 45-60分钟 | 完全初学者 |
| **EXECUTION_TRACE.md** | 逐步执行追踪 + 内存变化 | 60-90分钟 | 想理解细节的人 |
| **RUST_SYNTAX_CHEATSHEET.md** | 语法速查表 | 按需查阅 | 编码时查阅 |
| **LEARNING_RESOURCES.md** | 学习路线 + 资源导航 | 10-15分钟 | 规划学习计划 |
| **quick_start.sh** | 自动化脚本 | 即时 | 快速执行任务 |
| **README.md** | 官方项目说明 | 15分钟 | 了解项目背景 |

## 🎯 按目的选择文档

### "我是 Rust 初学者，从哪开始？"
1. ✅ 先读 **LEARNING_GUIDE.md** 的"Rust基础语法速成"部分
2. ✅ 再读"测试代码逐行讲解"部分
3. ✅ 运行 `bash quick_start.sh test`
4. ✅ 对照 **EXECUTION_TRACE.md** 理解执行过程
5. ✅ 需要查语法时用 **RUST_SYNTAX_CHEATSHEET.md**

### "我想理解执行流程"
1. ✅ 直接读 **EXECUTION_TRACE.md**
2. ✅ 每看到一个步骤，在源码中找到对应的行
3. ✅ 用 **RUST_SYNTAX_CHEATSHEET.md** 查不明白的语法

### "我想修改示例"
1. ✅ 运行 `bash quick_start.sh modify`
2. ✅ 选择一个修改建议
3. ✅ 按照建议修改代码
4. ✅ 运行 `RUST_LOG=info cargo test --release -- --nocapture`
5. ✅ 对照文档理解为什么结果会改变

### "我想快速查阅某个 Rust 语法"
1. ✅ 打开 **RUST_SYNTAX_CHEATSHEET.md**
2. ✅ 在"速查索引"部分按概念或错误类型搜索
3. ✅ 找到对应的代码例子

### "我想建立完整的学习计划"
1. ✅ 读 **LEARNING_RESOURCES.md** 的"学习路线"部分
2. ✅ 按照 4 天的计划逐步进行
3. ✅ 用清单追踪进度

## 🔑 核心概念快查

### Rust 语法

| 概念 | 关键字 | 查看文档 | 在投票机中的用法 |
|------|--------|---------|-----------------|
| 不可变变量 | `let` | CHEATSHEET§1 | `let polling_station_state = ...` |
| 可变变量 | `let mut` | CHEATSHEET§1 | `let mut polling_station = ...` |
| 借用 | `&` | CHEATSHEET§3 | `submit(&ballot1)` |
| 可变借用 | `&mut` | CHEATSHEET§3 | `pub fn vote(&mut self, ...)` |
| 结构体 | `struct` | CHEATSHEET§2 | `struct VotingMachineState { ... }` |
| 实现方法 | `impl` | CHEATSHEET§2 | `impl VotingMachineState { ... }` |
| 模式匹配 | `match` | CHEATSHEET§4 | `match x { Ok(...) => ... }` |
| 错误处理 | `Result<T, E>` | CHEATSHEET§4 | `.unwrap()`, `?` |
| 测试属性 | `#[test]` | CHEATSHEET§10 | `#[test] fn protocol() { ... }` |
| 宏 | `!` | CHEATSHEET§6 | `assert_eq!()`, `println!()` |

### 投票机概念

| 概念 | 相关文件 | 查看文档 | 关键代码 |
|------|---------|---------|---------|
| 投票机状态 | core/src/lib.rs | LEARNING_GUIDE§概览 | `VotingMachineState` |
| 投票逻辑 | core/src/lib.rs | EXECUTION_TRACE§①④ | `vote()` 方法 |
| 位图表示 | core/src/lib.rs | RUST_SYNTAX_CHEATSHEET§11 | `voter_bitfield: u32` |
| Host 端管理 | src/lib.rs | EXECUTION_TRACE§②-⑩ | `PollingStation` |
| Guest 初始化 | methods/guest/src/bin/init.rs | EXECUTION_TRACE§② | `env::commit()` |
| Guest 投票 | methods/guest/src/bin/submit.rs | EXECUTION_TRACE§③ | 状态转换 |
| 证明验证 | src/lib.rs | LEARNING_GUIDE§关键概念 | `receipt.verify()` |

## 📊 执行流程速览

```
步骤 | 操作      | 关键行动                    | 状态变化
─────┼──────────┼─────────────────────────────┼──────────────────
 ①  | init()   | 记录初始状态的哈希          | polls_open=true
 ②  | submit() | 投票者0投"否" → 不增加count | count=0, bits=1
 ③  | submit() | 投票者1投"是" → 增加count   | count=1, bits=3
 ④  | submit() | 投票者2投"是" → 增加count   | count=2, bits=7
 ⑤  | submit() | 投票者1重复 → 被拒          | count=2, bits=7
 ⑥  | submit() | 投票者3投"否" → 不增加count | count=2, bits=15
 ⑦  | freeze() | 关闭投票站                  | polls_open=false
 ⑧  | submit() | 投票者4投票 → 被拒          | count=2, bits=15
```

详细讲解见 **EXECUTION_TRACE.md** 的"执行追踪"部分。

## 🛠️ 常用命令

```bash
# 运行完整测试（推荐首次运行）
RUST_LOG=info cargo test --release -- --nocapture

# 快速测试（不优化编译）
cargo test

# 仅编译，不运行
cargo build

# 快速检查（不编译）
cargo check

# 清理构建文件
cargo clean

# 使用快速启动脚本
bash quick_start.sh test      # 完整测试
bash quick_start.sh test-fast # 快速测试
bash quick_start.sh doc       # 显示文档列表
bash quick_start.sh explain   # 显示架构图
bash quick_start.sh help      # 显示所有命令
```

详细命令说明见 **quick_start.sh**。

## 🎓 学习难度级别

### ⭐ 级别 1：概览（15 分钟）
- [ ] 运行测试看输出
- [ ] 看架构图理解三层结构（Core → Guest → Host）
- [ ] 理解"位图"的概念

**核心句子**：
> "投票机用一个 u32 的 32 个比特跟踪 32 个投票者是否已投票。"

---

### ⭐⭐ 级别 2：基础（1-2 小时）
- [ ] 理解 Rust 的所有权和借用
- [ ] 理解结构体和方法
- [ ] 跟踪执行到步骤④

**核心句子**：
> "Rust 中每个值有唯一的所有者，可以借用但不转移所有权。投票机用 &self 的不可变方法读取，&mut self 的可变方法修改状态。"

---

### ⭐⭐⭐ 级别 3：深入（2-3 小时）
- [ ] 完整理解执行追踪（步骤①-⑧）
- [ ] 理解 Guest 和 Host 的通信
- [ ] 理解 Result<T, E> 和 `?` 操作符

**核心句子**：
> "Host 将参数写入 Guest 的输入，Guest 执行业务逻辑并通过 stdout 返回新状态，Host 通过 journal 验证执行的正确性。"

---

### ⭐⭐⭐⭐ 级别 4：应用（3-5 小时）
- [ ] 修改示例添加新功能
- [ ] 理解零知识证明的价值
- [ ] 能够编写简单的 Rust 程序

**核心句子**：
> "零知识证明允许任何人验证某个计算确实按正确的规则执行过，而无需公开所有的中间数据。"

---

## 🎯 解题能力检测

### 你能回答这些问题吗？

**Level 1（概览）**
1. Q: 投票机最多支持多少个投票者？为什么？
   > A: 32 个。因为用 u32（32 位）表示，每位代表一个投票者。

2. Q: `voter_bitfield = 7` 表示什么？
   > A: 投票者 0, 1, 2 已投票（7 = 0b111）。

**Level 2（基础）**
3. Q: 为什么 `polling_station` 需要 `mut` 关键字？
   > A: 因为 `submit()` 方法接收 `&mut self` 参数，需要修改状态。

4. Q: `.unwrap()` 做了什么？
   > A: 提取 `Ok` 中的值，如果是 `Err` 则 panic。

**Level 3（深入）**
5. Q: Guest 为什么要计算和提交状态的哈希？
   > A: 证明计算的正确性，防止状态被篡改，任何人都可以验证。

6. Q: 为什么投票者重复投票会被拒绝？
   > A: 检查 `voter_bitfield & voter_mask == 0`，已投票的位为 1，条件失败。

**Level 4（应用）**
7. Q: 如何修改代码支持 64 个投票者？
   > A: 将 `voter_bitfield: u32` 改为 `voter_bitfield: u64`。

8. Q: 如何添加"否"票的统计？
   > A: 添加新字段 `count_no: u32`，在 `vote()` 中当 `!vote_yes` 时增加。

---

## ⚡ 5 分钟快速上手

```bash
# 1. 进入目录
cd examples/voting-machine

# 2. 运行测试（首次需要 5-10 分钟编译）
RUST_LOG=info cargo test --release -- --nocapture

# 3. 观察输出中的关键信息
# [INFO] init                          ← 步骤 ①
# [INFO] submit: Ballot { voter: 0, .. } ← 步骤 ②
# [INFO] Final vote count: 2           ← 最终结果

# 4. 打开 EXECUTION_TRACE.md
# 找到对应的步骤讲解

# 5. 对照 src/lib.rs 的测试函数阅读
```

## 🔍 按错误类型快速查找

| 错误信息 | 含义 | 查看 |
|---------|------|------|
| "value used after move" | 使用了已转移所有权的值 | CHEATSHEET§3, LEARNING_GUIDE§所有权 |
| "cannot borrow as mutable" | 没有标记 `mut` | CHEATSHEET§3, LEARNING_GUIDE§借用 |
| "expected &str, found String" | 类型不匹配 | CHEATSHEET§1 |
| "no method named" | 特征未导入 | CHEATSHEET§5 |
| "mismatched types" | 类型转换错误 | RUST_SYNTAX_CHEATSHEET§1 |

## 💾 文件列表

```
投票机示例/
├── 📖 LEARNING_GUIDE.md              (45-60分钟)  完整讲解
├── 📖 EXECUTION_TRACE.md             (60-90分钟)  执行追踪
├── 📖 RUST_SYNTAX_CHEATSHEET.md      (按需查阅)   语法速查
├── 📖 LEARNING_RESOURCES.md          (10-15分钟)  学习规划
├── 📖 QUICK_INDEX.md                 (这个文件)   快速索引
├── 🚀 quick_start.sh                 (即时使用)   自动化脚本
│
├── 📝 core/src/lib.rs                (主要业务逻辑)
├── 📝 src/lib.rs                     (Host 端)
├── 📝 methods/guest/src/bin/init.rs  (初始化 Guest)
├── 📝 methods/guest/src/bin/submit.rs (投票 Guest)
├── 📝 methods/guest/src/bin/freeze.rs (冻结 Guest)
│
└── 📝 README.md                      (官方说明)
```

## 🎊 学习成就解锁

按顺序完成以下任务解锁成就：

- 🔓 **学徒** — 运行了测试，看到了 `[INFO]` 日志
- 🔓 **初学者** — 理解了所有权和借用的概念
- 🔓 **学生** — 完整阅读了执行追踪，理解了每一步
- 🔓 **实践者** — 修改了至少 3 个不同的参数并观察结果
- 🔓 **开发者** — 添加了新功能（例如"否"票统计）或支持更多投票者
- 🔓 **大师** — 可以解释零知识证明为什么有用，以及如何应用到区块链

## 📞 获取帮助

- **文档问题**：查看对应的 .md 文件
- **命令问题**：运行 `bash quick_start.sh help`
- **代码问题**：对照 RUST_SYNTAX_CHEATSHEET.md 和源代码
- **执行问题**：阅读 EXECUTION_TRACE.md 的对应步骤
- **Rust 问题**：查看 The Rust Book (https://doc.rust-lang.org/book/)

---

**开始学习**: `bash quick_start.sh explain` 🚀

**有问题**: 查看 LEARNING_RESOURCES.md 的常见问题部分 ❓

**完成学习**: 使用清单追踪进度，直到所有项目打勾 ✅
