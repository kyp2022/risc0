# 投票机示例：实时执行追踪

这个文档会模拟 `cargo test` 的完整执行过程，并在每一步都展示状态变化。

## 预期的测试输出（`RUST_LOG=info cargo test -- --nocapture`）

```
running 1 test

[初始化阶段]
test tests::protocol ... 
    
    [TRACE] Compiling voting-machine-methods...
    [INFO] init
    [INFO] init  <-- 对应 src/lib.rs: PollingStation::init()
```

## 执行追踪（逐步分解）

### ← 主程序启动前的准备

```
$ RUST_LOG=info cargo test --release -- --nocapture
```

**发生了什么**：
1. **`RUST_LOG=info`**：设置日志级别，让 `tracing::info!()` 宏的输出可见
2. **`cargo test`**：
   - 编译主程序（src/lib.rs 中的 #[test] 函数）
   - 编译 voting-machine-methods 及其 guest
3. **`--release`**：优化编译（生成证明时更快，但编译本身较慢）
4. **`--nocapture`**：不隐藏 stdout，显示所有 println! 和 tracing 输出

---

### ① 执行 `tests::protocol()` — 创建初始状态

```rust
// 源代码（src/lib.rs）
#[test]
fn protocol() {
    let polling_station_state = VotingMachineState {
        polls_open: true,        // 投票站开放
        voter_bitfield: 0,       // 还没人投票：0b00000000
        count: 0,                // "是"票数：0
    };

    let mut polling_station = PollingStation::new(polling_station_state);
```

**内存状态**（执行后）：
```
┌─ polling_station ──────────────────────┐
│ state:                                 │
│  ├─ polls_open: true                   │
│  ├─ voter_bitfield: 0 (0b00000000)     │
│  └─ count: 0                           │
└────────────────────────────────────────┘
```

---

### ② 调用 `polling_station.init()`

**源代码追踪**：
```rust
let init_msg = polling_station.init().unwrap();
```

**执行步骤**（host 端）：

```rust
// src/lib.rs: PollingStation::init()
pub fn init(&self) -> Result<InitMessage> {
    tracing::info!("init");  // ← 日志输出：[INFO] init
    
    // Step 1: 构建执行环境，将状态写入 guest 输入
    let env = ExecutorEnv::builder()
        .write(&self.state)?
        .build()?;
    
    // Step 2: 获取证明器
    let prover = default_prover();
    
    // Step 3: 执行 guest 程序（init.rs），得到证明收据
    let receipt = prover.prove(env, INIT_ELF)?.receipt;
    
    // Step 4: 包装返回
    Ok(InitMessage { receipt })
}
```

**Guest 执行**（`methods/guest/src/bin/init.rs`）：

```rust
#![no_main]
#![no_std]

risc0_zkvm::guest::entry!(main);

fn main() {
    // Step 1: Guest 读取 Host 写入的状态
    let state: VotingMachineState = env::read();
    //   state = VotingMachineState { 
    //       polls_open: true, 
    //       voter_bitfield: 0, 
    //       count: 0 
    //   }
    
    // Step 2: 序列化状态为字节数组
    let state_bytes = to_vec(&state).unwrap();
    //   state_bytes = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, ...]
    //   （具体字节取决于序列化格式，这里只是示意）
    
    // Step 3: 计算状态的 SHA256 哈希
    let state_hash = *Impl::hash_words(&state_bytes);
    //   state_hash = Digest { /* 32 字节的哈希值 */ }
    
    // Step 4: 提交初始化记录到 journal
    env::commit(&InitializeVotingMachineCommit {
        polls_open: true,
        voter_bitfield: 0,
        state: state_hash,
    });
    // ← 这个 commit 被记录在 Receipt 的 journal 中
}
```

**证明过程**（Prover）：
```
输入: state = VotingMachineState { polls_open: true, ... }
      ↓
执行 guest 代码（可被证明）
      ↓
生成证明 + 输出
      ↓
Receipt {
    journal: [InitializeVotingMachineCommit { ... }],
    proof: <零知识证明>,
    ...
}
```

**Host 接收证明**：
```rust
Ok(InitMessage { receipt })
```

**内存状态**（执行后）：
```
┌─ init_msg ─────────────────────┐
│ receipt:                       │
│  ├─ journal: [...commit...]    │
│  ├─ proof: <ZK 证明>           │
│  └─ other metadata             │
└────────────────────────────────┘
```

**日志输出**：
```
[INFO] init
```

---

### ③ 调用 `polling_station.submit(&ballot1)` — 投票者0投"否"票

**源代码**：
```rust
let ballot1 = Ballot {
    voter: 0,
    vote_yes: false,  // 投"否"票
};
let ballot_msg1 = polling_station.submit(&ballot1).unwrap();
```

**Host 端执行**（`src/lib.rs: PollingStation::submit()`）：

```rust
pub fn submit(&mut self, ballot: &Ballot) -> Result<SubmitBallotMessage> {
    tracing::info!("submit: {:?}", ballot);  // ← [INFO] submit: Ballot { voter: 0, vote_yes: false }
    
    // Step 1: 创建提交参数
    let params = SubmitBallotParams::new(
        self.state.clone(),  // 当前状态：{ polls_open: true, voter_bitfield: 0, count: 0 }
        ballot.clone(),      // 选票：{ voter: 0, vote_yes: false }
    );
    
    // Step 2: 用于接收 guest 的输出
    let mut output = Vec::new();
    
    // Step 3: 构建执行环境
    let env = ExecutorEnv::builder()
        .write(&params)?          // 写入参数
        .stdout(&mut output)      // 设置 stdout（guest 会写入新状态）
        .build()?;
    
    // Step 4: 运行证明器
    let prover = default_prover();
    let receipt = prover.prove(env, SUBMIT_ELF)?.receipt;
    
    // Step 5: 从 output 读取新状态并更新本地
    self.state = from_slice(&output)?;
    //   self.state 更新为 guest 返回的新状态
    
    Ok(SubmitBallotMessage { receipt })
}
```

**Guest 执行**（`methods/guest/src/bin/submit.rs`）：

```rust
fn main() {
    // Step 1: 读取 params
    let params: SubmitBallotParams = env::read();
    //   params.state = { polls_open: true, voter_bitfield: 0, count: 0 }
    //   params.ballot = { voter: 0, vote_yes: false }
    
    // Step 2: 处理投票逻辑（在 core 中实现）
    let result = params.process();
    //   ↓ 调用 SubmitBallotParams::process()
    //   ↓ 内部调用 state.vote(0, false)
    
    //   state.vote(0, false) 的执行：
    //     let voter_mask = 1 << 0;        // 0b00000001
    //     if polls_open && (bitfield & mask == 0) {  // true && (0 & 1 == 0)? YES
    //         voter_bitfield |= mask;    // 0 | 1 = 1（0b00000001）
    //         if vote_yes { count += 1 } // false，所以 count 不变
    //         true
    //     }
    //   返回：SubmitBallotResult {
    //       state: { polls_open: true, voter_bitfield: 1, count: 0 },
    //       vote_counted: true,
    //       vote_yes: false
    //   }
    
    // Step 3: 将新状态写回 host（通过 stdout）
    env::write(&result.state);
    //   写入序列化的新状态到 stdout
    
    // Step 4: 计算旧状态哈希
    let old_state_bytes = to_vec(&params.state).unwrap();
    let old_state_hash = *Impl::hash_words(&old_state_bytes);
    //   旧状态哈希（voter_bitfield=0 时）
    
    // Step 5: 计算新状态哈希
    let new_state_bytes = to_vec(&result.state).unwrap();
    let new_state_hash = *Impl::hash_words(&new_state_bytes);
    //   新状态哈希（voter_bitfield=1 时）
    
    // Step 6: 提交证明数据
    env::commit(&SubmitBallotCommit {
        old_state: old_state_hash,      // 旧状态的哈希
        new_state: new_state_hash,      // 新状态的哈希
        polls_open: true,
        voter_bitfield: 1,              // 新状态的 voter_bitfield
        voter: 0,
        vote_yes: false,
        vote_counted: true,             // 投票成功
    });
}
```

**Host 接收并更新**：
```rust
// 从 output 读取新状态
self.state = from_slice(&output)?;
//   self.state 现在 = { polls_open: true, voter_bitfield: 1, count: 0 }
```

**内存状态变化**：
```
投票前：
┌─ polling_station.state ────────────────┐
│ polls_open: true                       │
│ voter_bitfield: 0 (0b00000000)         │
│ count: 0                               │
└────────────────────────────────────────┘

投票后：
┌─ polling_station.state ────────────────┐
│ polls_open: true                       │
│ voter_bitfield: 1 (0b00000001) ← 改变  │
│ count: 0                               │
└────────────────────────────────────────┘
```

**日志输出**：
```
[INFO] submit: Ballot { voter: 0, vote_yes: false }
```

---

### ④ 调用 `polling_station.submit(&ballot2)` — 投票者1投"是"票

**源代码**：
```rust
let ballot2 = Ballot {
    voter: 1,
    vote_yes: true,  // 投"是"票
};
let ballot_msg2 = polling_station.submit(&ballot2).unwrap();
```

**关键步骤**（简化版）：

```rust
// Host 发送参数
params = {
    state: { polls_open: true, voter_bitfield: 1, count: 0 },  // ← 来自上一步的更新
    ballot: { voter: 1, vote_yes: true }
}

// Guest 执行 state.vote(1, true)
let voter_mask = 1 << 1;           // 0b00000010
if polls_open && (bitfield & mask == 0) {  // true && (1 & 2 == 0)? YES
    voter_bitfield |= mask;        // 1 | 2 = 3（0b00000011）
    if vote_yes { count += 1 }      // YES，count = 0 + 1 = 1
    true
}

// Host 更新
self.state = { polls_open: true, voter_bitfield: 3, count: 1 }
```

**内存状态**：
```
┌─ polling_station.state ────────────────┐
│ polls_open: true                       │
│ voter_bitfield: 3 (0b00000011)         │
│ count: 1 ← 增加了，因为 vote_yes=true  │
└────────────────────────────────────────┘
```

**日志输出**：
```
[INFO] submit: Ballot { voter: 1, vote_yes: true }
```

---

### ⑤ 调用 `polling_station.submit(&ballot3)` — 投票者2投"是"票

**结果**：
```
state.vote(2, true)
voter_mask = 1 << 2 = 0b00000100
voter_bitfield = 3 | 4 = 7（0b00000111）
count = 1 + 1 = 2

polling_station.state = { polls_open: true, voter_bitfield: 7, count: 2 }
```

**日志输出**：
```
[INFO] submit: Ballot { voter: 2, vote_yes: true }
```

---

### ⑥ 调用 `polling_station.submit(&ballot4)` — 投票者1重复投票（失败）

**源代码**：
```rust
let ballot4 = Ballot {
    voter: 1,         // 同一个投票者
    vote_yes: false,
};
let ballot_msg4 = polling_station.submit(&ballot4).unwrap();
```

**Guest 执行**：
```rust
// state.vote(1, false)
let voter_mask = 1 << 1;           // 0b00000010
// 检查：polls_open? YES
//      bitfield & mask == 0? 7 & 2 == 0? 0b0111 & 0b0010 = 0b0010 ≠ 0? NO！
if polls_open && (bitfield & mask == 0) {  // true && false? 条件失败！
    // ... 不执行 ...
    false  // 返回 false
}

// 状态不变
polling_station.state = { polls_open: true, voter_bitfield: 7, count: 2 }
```

**关键点**：投票者1已在第④步的位图中被标记（bit[1]=1），所以检查失败，投票被拒绝。

**日志输出**：
```
[INFO] submit: Ballot { voter: 1, vote_yes: false }
```

---

### ⑦ 调用 `polling_station.submit(&ballot5)` — 投票者3投"否"票

**结果**：
```
state.vote(3, false)
voter_bitfield = 7 | 8 = 15（0b00001111）
count = 2（不增加，因为 vote_yes=false）

polling_station.state = { polls_open: true, voter_bitfield: 15, count: 2 }
```

**日志输出**：
```
[INFO] submit: Ballot { voter: 3, vote_yes: false }
```

---

### ⑧ 调用 `polling_station.freeze()` — 冻结投票站

**源代码**：
```rust
let close_msg = polling_station.freeze().unwrap();
```

**Host 端执行**（`src/lib.rs: PollingStation::freeze()`）：

```rust
pub fn freeze(&mut self) -> Result<FreezeStationMessage> {
    tracing::info!("freeze");  // ← [INFO] freeze
    
    let params = FreezeVotingMachineParams::new(self.state.clone());
    //   params.state = { polls_open: true, voter_bitfield: 15, count: 2 }
    
    let mut output = Vec::new();
    let env = ExecutorEnv::builder()
        .write(&params)?
        .stdout(&mut output)
        .build()?;
    
    let prover = default_prover();
    let receipt = prover.prove(env, FREEZE_ELF)?.receipt;
    
    let result: FreezeVotingMachineResult = from_slice(&output)?;
    self.state = result.state;
    //   self.state 现在 = { polls_open: false, voter_bitfield: 15, count: 2 }
    
    Ok(FreezeStationMessage { receipt })
}
```

**Guest 执行**（`methods/guest/src/bin/freeze.rs`）：

```rust
fn main() {
    let params: FreezeVotingMachineParams = env::read();
    //   params.state = { polls_open: true, voter_bitfield: 15, count: 2 }
    
    let result = params.process();
    //   ↓ 调用 FreezeVotingMachineParams::process()
    //   ↓ 将 polls_open 设为 false
    //   返回：FreezeVotingMachineResult {
    //       state: { polls_open: false, voter_bitfield: 15, count: 2 }
    //   }
    
    env::write(&result.state);
    //   写入新状态到 stdout
    
    // 计算哈希并 commit
    let old_state_hash = ...;  // polls_open=true 时的哈希
    let new_state_hash = ...;  // polls_open=false 时的哈希
    
    env::commit(&FreezeVotingMachineCommit {
        old_state: old_state_hash,
        new_state: new_state_hash,
        polls_open: false,
        voter_bitfield: 15,
        count: 2,              // 最终票数
    });
}
```

**内存状态**：
```
冻结前：
┌─ polling_station.state ─────────────────┐
│ polls_open: true                        │
│ voter_bitfield: 15 (0b00001111)         │
│ count: 2                                │
└─────────────────────────────────────────┘

冻结后：
┌─ polling_station.state ─────────────────┐
│ polls_open: false ← 改变！              │
│ voter_bitfield: 15 (0b00001111)         │
│ count: 2                                │
└─────────────────────────────────────────┘
```

**日志输出**：
```
[INFO] freeze
```

---

### ⑨ 调用 `polling_station.submit(&ballot6)` — 投票站关闭后投票（失败）

**源代码**：
```rust
let ballot6 = Ballot {
    voter: 4,
    vote_yes: true,
};
let ballot_msg6 = polling_station.submit(&ballot6).unwrap();
```

**Guest 执行**：
```rust
// state.vote(4, true)
let voter_mask = 1 << 4;           // 0b00010000
// 检查：polls_open? NO（已在⑧冻结）
if polls_open && (bitfield & mask == 0) {  // false && ...? 条件立即失败！
    // ... 不执行 ...
    false  // 返回 false
}

// 状态不变
polling_station.state = { polls_open: false, voter_bitfield: 15, count: 2 }
```

**关键点**：投票站已关闭（`polls_open=false`），第一个条件就失败了。

**日志输出**：
```
[INFO] submit: Ballot { voter: 4, vote_yes: true }
```

---

### ⑩ 最终断言与验证

**源代码**：
```rust
    assert_eq!(polling_station.state.count, 2);

    let init_state = init_msg.verify_and_get_commit();
    let ballot_commit1 = ballot_msg1.verify_and_get_commit();
    // ... 更多验证 ...
    let ballot_commit6 = ballot_msg6.verify_and_get_commit();

    tracing::info!("initial commit: {:?}", init_state);
    tracing::info!("ballot 1: {:?}", ballot1);
    tracing::info!("ballot 1 commit: {:?}", ballot_commit1);
    // ... 更多日志 ...
    tracing::info!("Final vote count: {:?}", polling_station.state.count);
}
```

**执行**：

```rust
// Step 1: 断言最终票数
assert_eq!(polling_station.state.count, 2);
//   polling_station.state.count == 2? YES ✓ PASS

// Step 2: 验证所有收据（通过零知识证明验证其正确性）
let init_state = init_msg.verify_and_get_commit();
//   ↓ 调用 init_msg.receipt.verify(INIT_ID)?
//   ↓ 验证 receipt 的有效性（检查证明）
//   ↓ 调用 init_msg.get_state()
//   ↓ 从 receipt.journal 解码 InitializeVotingMachineCommit
//   返回：Ok(InitializeVotingMachineCommit { ... })

// 同样验证其他 receipts...

// Step 3: 日志输出（展示最终结果）
tracing::info!("initial commit: {:?}", init_state);
//   输出：[INFO] initial commit: Ok(InitializeVotingMachineCommit { polls_open: true, voter_bitfield: 0, state: Digest(...) })

tracing::info!("Final vote count: {:?}", polling_station.state.count);
//   输出：[INFO] Final vote count: 2
```

---

## 完整输出（预期结果）

```
running 1 test
test tests::protocol ... [INFO] init
[INFO] submit: Ballot { voter: 0, vote_yes: false }
[INFO] submit: Ballot { voter: 1, vote_yes: true }
[INFO] submit: Ballot { voter: 2, vote_yes: true }
[INFO] submit: Ballot { voter: 1, vote_yes: false }
[INFO] submit: Ballot { voter: 3, vote_yes: false }
[INFO] freeze
[INFO] submit: Ballot { voter: 4, vote_yes: true }
[INFO] initial commit: Ok(InitializeVotingMachineCommit { polls_open: true, voter_bitfield: 0, state: Digest(...) })
[INFO] ballot 1: Ballot { voter: 0, vote_yes: false }
[INFO] ballot 1 commit: Ok(SubmitBallotCommit { old_state: Digest(...), new_state: Digest(...), polls_open: true, voter_bitfield: 1, voter: 0, vote_yes: false, vote_counted: true })
[INFO] ballot 2: Ballot { voter: 1, vote_yes: true }
[INFO] ballot 2 commit: Ok(SubmitBallotCommit { ..., voter: 1, vote_yes: true, vote_counted: true })
[INFO] ballot 3: Ballot { voter: 2, vote_yes: true }
[INFO] ballot 3 commit: Ok(SubmitBallotCommit { ..., voter: 2, vote_yes: true, vote_counted: true })
[INFO] ballot 4: Ballot { voter: 1, vote_yes: false }
[INFO] ballot 4 commit: Ok(SubmitBallotCommit { ..., voter: 1, vote_yes: false, vote_counted: false })
[INFO] ballot 5: Ballot { voter: 3, vote_yes: false }
[INFO] ballot 5 commit: Ok(SubmitBallotCommit { ..., voter: 3, vote_yes: false, vote_counted: true })
[INFO] freeze commit: Ok(FreezeVotingMachineCommit { old_state: Digest(...), new_state: Digest(...), polls_open: false, voter_bitfield: 15, count: 2 })
[INFO] ballot 6: Ballot { voter: 4, vote_yes: true }
[INFO] ballot 6 commit: Ok(SubmitBallotCommit { ..., voter: 4, vote_yes: true, vote_counted: false })
[INFO] Final vote count: 2

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 状态变化时间线（总结表）

| 步骤 | 操作 | voter | vote_yes | 结果 | 新状态 |
|------|------|-------|----------|------|--------|
| ① | init | - | - | 记录初始 | polls=T, bits=0, count=0 |
| ② | submit | 0 | F | ✓ | polls=T, bits=1, count=0 |
| ③ | submit | 1 | T | ✓ | polls=T, bits=3, count=1 |
| ④ | submit | 2 | T | ✓ | polls=T, bits=7, count=2 |
| ⑤ | submit | 1 | F | ✗ 重复 | polls=T, bits=7, count=2 |
| ⑥ | submit | 3 | F | ✓ | polls=T, bits=15, count=2 |
| ⑦ | freeze | - | - | 冻结 | polls=F, bits=15, count=2 |
| ⑧ | submit | 4 | T | ✗ 关闭 | polls=F, bits=15, count=2 |

---

## 关键观察

1. **位图的作用**：`voter_bitfield` 保证每个投票者只能投一次
   - 1 = 0b0001（投票者0已投）
   - 3 = 0b0011（投票者0, 1已投）
   - 7 = 0b0111（投票者0, 1, 2已投）
   - 15 = 0b1111（投票者0, 1, 2, 3已投）

2. **count 的含义**：只统计 `vote_yes=true` 的票
   - 投票者0投"否" → count 不变（0）
   - 投票者1投"是" → count 增加到 1
   - 投票者2投"是" → count 增加到 2
   - 投票者3投"否" → count 不变（2）

3. **证明的价值**：
   - 每个操作都有 receipt（包含 journal 和 proof）
   - 任何人都可以验证整个流程的正确性
   - 投票者可以证明"我投过票"而不公开投票内容

4. **失败的案例**：
   - 重复投票被拒（投票者1的第二次投票）
   - 关闭后投票被拒（投票者4的投票）

---

## 如何在本地运行并观察

```bash
cd /Users/ppg/Desktop/zkvm/risc0/examples/voting-machine

# 完整的输出（包括所有日志）
RUST_LOG=info cargo test --release -- --nocapture

# 只运行特定测试
RUST_LOG=info cargo test --release protocol -- --nocapture

# 不优化编译（更快，但证明更慢）
RUST_LOG=info cargo test -- --nocapture
```

---

## 下一步实验想法

1. **修改投票内容**：改变 ballot2-6 的 voter 和 vote_yes 值，观察 count 和 bitfield 如何变化

2. **添加更多投票者**：增加 ballot7, ballot8... 观察到 32 的限制

3. **改变冻结时机**：在不同位置调用 freeze()，观察影响

4. **验证失败场景**：故意修改某个 commit，看 verify 是否失败

5. **性能测试**：计时每个操作，看 prover 的开销
