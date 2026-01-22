# 三种 Commit 的对比讲解

> **深入理解三个 Commit 结构的设计目的与区别**

---

## 快速对比表

| Commit 类型 | 何时产生 | 主要作用 | 关键字段 |
|----------|---------|--------|--------|
| `InitializeVotingMachineCommit` | `init()` | 记录初始状态 | `state`（哈希） |
| `SubmitBallotCommit` | `submit()` | 记录状态转换 | `old_state`, `new_state` |
| `FreezeVotingMachineCommit` | `freeze()` | 记录最终冻结 | `count`（最终票数） |

---

## 详细对比

### 1. InitializeVotingMachineCommit（初始化）

**定义：**
```rust
pub struct InitializeVotingMachineCommit {
    pub polls_open: bool,       // ← 初始状态：投票站是否开放
    pub voter_bitfield: u32,    // ← 初始状态：投票者位图
    pub state: Digest,          // ← 初始状态的哈希
}
```

**何时产生：**
```
polling_station.init()
    ↓
Guest 程序 init.rs 执行 main()
    ↓
env::commit(&InitializeVotingMachineCommit { ... })
    ↓
commit 写入 Journal
```

**作用：**
- 记录投票机启动时的初始状态
- 证明：投票机确实以这个状态开始
- 用于：后续所有操作的基准点

**现实类比：**
```
如同法庭记录：
  "投票机在 2024-01-04 12:00:00 初始化"
  "初始状态：投票站开放，0 人投票"
  "证明文件编号：# INIT-20240104-001"
```

---

### 2. SubmitBallotCommit（投票提交）

**定义：**
```rust
pub struct SubmitBallotCommit {
    pub old_state: Digest,      // ← 投票前的状态哈希
    pub new_state: Digest,      // ← 投票后的状态哈希
    pub polls_open: bool,       // ← 投票时投票站是否开放
    pub voter_bitfield: u32,    // ← 投票后的位图
    pub voter: u32,             // ← 投票者 ID
    pub vote_yes: bool,         // ← 是否投"是"票
    pub vote_counted: bool,     // ← 投票是否被计入
}
```

**何时产生：**
```
polling_station.submit(&ballot)
    ↓
Guest 程序 submit.rs 执行 main()
    ↓
env::commit(&SubmitBallotCommit { ... })
    ↓
commit 写入 Journal
```

**作用：**
- 记录一张选票的处理过程
- 证明：状态确实从 old 转换到 new
- 证明：这个投票者和票数都被记录
- 证明：投票是否成功计入（防重复投票）

**现实类比：**
```
如同收据：
  "投票者 3 在 12:05:30 投票"
  "投票内容：YES"
  "投票前状态哈希：0xABCD..."
  "投票后状态哈希：0x1234..."
  "是否成功：YES（成功）"
  "当前投票人数位图：0b1111（前4人已投）"
```

**关键差异（与 InitializeVotingMachineCommit）：**
```
InitializeVotingMachineCommit:
  - 只记录初始状态
  - 一个状态哈希 (state)
  - 不涉及投票者和票数

SubmitBallotCommit:
  - 记录状态转换
  - 两个状态哈希 (old_state, new_state)
  - 包含投票者信息 (voter, vote_yes)
  - 包含成功标志 (vote_counted)
```

---

### 3. FreezeVotingMachineCommit（冻结）

**定义：**
```rust
pub struct FreezeVotingMachineCommit {
    pub old_state: Digest,      // ← 冻结前的状态哈希
    pub new_state: Digest,      // ← 冻结后的状态哈希
    pub polls_open: bool,       // ← 冻结后应为 false
    pub voter_bitfield: u32,    // ← 投票者位图（最终）
    pub count: u32,             // ← 最终票数（关键！）
}
```

**何时产生：**
```
polling_station.freeze()
    ↓
Guest 程序 freeze.rs 执行 main()
    ↓
env::commit(&FreezeVotingMachineCommit { ... })
    ↓
commit 写入 Journal
```

**作用：**
- 记录投票机冻结时的最终状态
- 证明：投票站确实关闭（polls_open: false）
- 证明：最终票数是多少（count）
- 用于：宣布选举结果

**现实类比：**
```
如同公告：
  "投票在 2024-01-04 17:00:00 结束"
  "投票站已关闭"
  "最终"是"票数：2"
  "参与投票者：4 人（位图：0b1111）"
  "证明文件编号：# FREEZE-20240104-001"
```

**关键差异（与其他两个）：**
```
InitializeVotingMachineCommit:
  - 用于起始
  - 记录初始条件

SubmitBallotCommit:
  - 用于过程
  - 记录每一次投票
  - 可能有多个（一次投票一个）

FreezeVotingMachineCommit:
  - 用于结束
  - 只有一个（最后的冻结）
  - 包含最终票数 (count)
```

---

## 完整流程示意

### 时间线与三种 Commit

```
时间 → 

[初始化]
  polling_station.init()
  ↓
  InitializeVotingMachineCommit {
    polls_open: true,
    voter_bitfield: 0,
    state: Hash1
  }
  ↓
  Journal 中的 Entry 0

[投票1] 投票者0投否票
  polling_station.submit(&Ballot { voter: 0, vote_yes: false })
  ↓
  SubmitBallotCommit {
    old_state: Hash1,
    new_state: Hash2,
    polls_open: true,
    voter_bitfield: 0b0001,      // ← 投票者0已投票
    voter: 0,
    vote_yes: false,
    vote_counted: true,
  }
  ↓
  Journal 中的 Entry 1

[投票2] 投票者1投是票
  polling_station.submit(&Ballot { voter: 1, vote_yes: true })
  ↓
  SubmitBallotCommit {
    old_state: Hash2,
    new_state: Hash3,
    polls_open: true,
    voter_bitfield: 0b0011,      // ← 投票者0、1已投票
    voter: 1,
    vote_yes: true,
    vote_counted: true,
  }
  ↓
  Journal 中的 Entry 2

[投票3] 投票者2投是票
  ... 类似 ...
  ↓
  Journal 中的 Entry 3

[冻结]
  polling_station.freeze()
  ↓
  FreezeVotingMachineCommit {
    old_state: Hash3 或 Hash4（最后一次投票的结果）,
    new_state: Hash_frozen,
    polls_open: false,           // ← 已关闭
    voter_bitfield: 0b0111,      // ← 最终：3人投票
    count: 2,                    // ← 最终结果："是"票2张
  }
  ↓
  Journal 中的 Entry N

【完整 Receipt（证明）】
receipt {
  proof: <密码学证明>,
  journal: [
    Entry 0: InitializeVotingMachineCommit { ... }
    Entry 1: SubmitBallotCommit { ... }
    Entry 2: SubmitBallotCommit { ... }
    Entry 3: SubmitBallotCommit { ... }
    Entry N: FreezeVotingMachineCommit { ... }
  ]
}
```

---

## 为什么要设计成三种不同的结构？

### 问题 1：为什么不用一个统一的 Commit？

**不好的设计：**
```rust
pub struct UnifiedCommit {
    pub step: String,           // "init" / "submit" / "freeze"
    pub init_state: Option<...>,
    pub submit_info: Option<...>,
    pub freeze_info: Option<...>,
    // ... 一堆可选字段 ...
}
```

**问题：**
- 太复杂，易出错
- 冗余数据（大量 Option）
- 不清晰：哪些字段在哪个步骤有效

**好的设计（实际采用）：**
```rust
pub struct InitializeVotingMachineCommit { ... }  // 只有初始化需要的字段
pub struct SubmitBallotCommit { ... }             // 只有投票需要的字段
pub struct FreezeVotingMachineCommit { ... }      // 只有冻结需要的字段
```

**优点：**
- 清晰：每个结构体职责明确
- 高效：没有冗余字段
- 类型安全：编译器可以检查

### 问题 2：为什么投票（submit）需要两个哈希？

**初始化只需要一个哈希：**
```rust
state: Digest  // 初始状态的哈希
```

**为什么？因为初始化前没有"前置状态"**

**投票需要两个哈希：**
```rust
old_state: Digest,  // 投票前的状态
new_state: Digest,  // 投票后的状态
```

**为什么？为了证明状态转换的正确性**

```
验证过程：
  1. 从 Journal 中读取 Entry i（投票 i）的 SubmitBallotCommit
  2. 获取 old_state = 0xABCD...
  3. 从 Entry i-1（投票 i-1）的 new_state 应该是 0xABCD...
  4. 如果不相等，说明有人篡改了！
  
  这样可以验证整条链：
    init_state → submit1_new → submit2_new → ... → freeze_new
    
  任何篡改都会导致哈希不匹配，验证失败
```

### 问题 3：为什么冻结需要单独的 Commit？

**不能用最后一次投票的 SubmitBallotCommit？**

**不能，因为：**
```
假设没有冻结 Commit，而是用最后一次投票的 SubmitBallotCommit

问题：
  1. 如果最后一票是投票者 0 投的，那么 Commit 会包含 voter: 0
  2. Verifier 看到这个 Commit，可能误以为总共只有一个人投票
  3. 混淆：是"一个人投的最后一票"还是"总共一个人投票"

解决：用单独的 FreezeVotingMachineCommit
  1. 明确记录最终状态
  2. polls_open: false 表示已冻结
  3. count: N 表示最终票数
  4. 清晰区分投票阶段和冻结阶段
```

---

## 数据验证链（Verification Chain）

### 完整验证过程

```
Receipt 中的 Journal：
[
  InitializeVotingMachineCommit {
    state: H_0,
  },
  SubmitBallotCommit {
    old_state: H_0,        ← 应该等于前面的 state
    new_state: H_1,        ← 这个投票的结果
  },
  SubmitBallotCommit {
    old_state: H_1,        ← 应该等于前面的 new_state
    new_state: H_2,
  },
  SubmitBallotCommit {
    old_state: H_2,        ← 应该等于前面的 new_state
    new_state: H_3,
  },
  FreezeVotingMachineCommit {
    old_state: H_3,        ← 应该等于前面的 new_state
    new_state: H_frozen,
  },
]

验证算法（伪代码）：
prev_hash = H_0
for i = 1 to N:
  commit = journal[i]
  if commit.old_state != prev_hash:
    return ERROR("链断裂！可能被篡改")
  prev_hash = commit.new_state
return OK("链完整，所有操作合法")
```

### 为什么这样可以防止篡改？

```
攻击者尝试篡改 Entry 3：
  原：SubmitBallotCommit { old_state: H_2, new_state: H_3, ... }
  改：SubmitBallotCommit { old_state: H_2, new_state: H_3', ... }
       （修改了 new_state，试图隐藏一张投票）

问题：
  Entry 4 仍然期望 old_state: H_3
  但现在 Entry 3 的 new_state: H_3'
  H_3' ≠ H_3 → 验证失败！

结论：
  攻击者必须修改所有后续 Entry
  但 Entry 中的哈希值由 Guest 在 ZKVM 中计算并记录
  ZKVM 内的执行是不可篡改的（密码学保证）
  → 无法篡改！
```

---

## Rust 结构体语法对比

### 三种 Commit 的定义对比

```rust
// 1. 初始化（简单）
#[derive(Debug, Deserialize, Serialize)]
pub struct InitializeVotingMachineCommit {
    pub polls_open: bool,        // 1 byte
    pub voter_bitfield: u32,     // 4 bytes
    pub state: Digest,           // 32 bytes
}
// 总大小：37 bytes

// 2. 投票提交（复杂）
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitBallotCommit {
    pub old_state: Digest,       // 32 bytes
    pub new_state: Digest,       // 32 bytes
    pub polls_open: bool,        // 1 byte
    pub voter_bitfield: u32,     // 4 bytes
    pub voter: u32,              // 4 bytes
    pub vote_yes: bool,          // 1 byte
    pub vote_counted: bool,      // 1 byte
}
// 总大小：75 bytes

// 3. 冻结（中等复杂）
#[derive(Debug, Deserialize, Serialize)]
pub struct FreezeVotingMachineCommit {
    pub old_state: Digest,       // 32 bytes
    pub new_state: Digest,       // 32 bytes
    pub polls_open: bool,        // 1 byte
    pub voter_bitfield: u32,     // 4 bytes
    pub count: u32,              // 4 bytes
}
// 总大小：73 bytes
```

**为什么 SubmitBallotCommit 最复杂？**

因为它需要记录：
- 状态转换（old/new）
- 投票内容（voter, vote_yes）
- 投票结果（vote_counted）
- 当前状态（polls_open, voter_bitfield）

其他两个只需要记录关键状态即可。

---

## 在测试中的体现

### test 中的调用顺序

```rust
#[test]
fn protocol() {
    let mut polling_station = PollingStation::new(initial_state);
    
    // 第 1 步：初始化
    let init_msg = polling_station.init().unwrap();
    // ↓ journal 中：InitializeVotingMachineCommit
    
    // 第 2 步：投票 1
    let ballot_msg1 = polling_station.submit(&ballot1).unwrap();
    // ↓ journal 中：SubmitBallotCommit { old_state: H_init, new_state: H_1 }
    
    // 第 3 步：投票 2
    let ballot_msg2 = polling_station.submit(&ballot2).unwrap();
    // ↓ journal 中：SubmitBallotCommit { old_state: H_1, new_state: H_2 }
    
    // ... 更多投票 ...
    
    // 第 N 步：冻结
    let close_msg = polling_station.freeze().unwrap();
    // ↓ journal 中：FreezeVotingMachineCommit { old_state: H_prev, new_state: H_frozen }
    
    // 验证
    let init_commit = init_msg.verify_and_get_commit().unwrap();
    let ballot1_commit = ballot_msg1.verify_and_get_commit().unwrap();
    // ... 所有 commit 都被验证 ...
}
```

---

## 总结对比

| 方面 | InitializeVotingMachineCommit | SubmitBallotCommit | FreezeVotingMachineCommit |
|------|--------------------------------|--------------------|---------------------------|
| **调用函数** | `init()` | `submit()` | `freeze()` |
| **何时产生** | 投票开始前 | 每次投票时 | 投票结束时 |
| **产生次数** | 1 次 | N 次（N = 投票数） | 1 次 |
| **主要字段** | 初始状态 | 状态转换 + 投票信息 | 最终状态 |
| **哈希字段** | 1 个 (state) | 2 个 (old/new) | 2 个 (old/new) |
| **关键信息** | `polls_open`, `voter_bitfield` | `voter`, `vote_yes`, `vote_counted` | `count` |
| **用途** | 证明起点 | 证明过程 | 证明终点 |
| **验证方式** | 单独验证 | 链验证（与前一个对比） | 链验证 + 最终检查 |

---

## 后续学习

- 详细 init 流程：见 `LEARNING_GUIDE_InitializeCommit.md`
- 详细 submit 流程：见 `LEARNING_GUIDE_SubmitBallot.md`（待创建）
- 完整运行体验：`RUST_LOG=info cargo test --release -- --nocapture`
