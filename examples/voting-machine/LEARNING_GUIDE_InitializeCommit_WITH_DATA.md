# InitializeVotingMachineCommit 执行流程 - 实际数据演示

> **这个文档展示每一步执行时，内存中的实际数据变化**

---

## 完整执行过程（逐步数据流）

### 【第 1 步】Host 创建初始状态

**源代码（src/lib.rs 中的 test）：**
```rust
let polling_station_state = VotingMachineState {
    polls_open: true,
    voter_bitfield: 0,
    count: 0,
};
```

**内存状态：**
```
┌──────────────────────────────────┐
│  polling_station_state           │
├──────────────────────────────────┤
│  polls_open: true                │  ← 1 byte，值为 0x01
│  voter_bitfield: 0               │  ← 4 bytes，值为 0x00000000
│  count: 0                        │  ← 4 bytes，值为 0x00000000
└──────────────────────────────────┘
总共：9 字节
```

**二进制表示：**
```
polls_open        voter_bitfield   count
      ↓                 ↓             ↓
    0x01         0x00 0x00 0x00   0x00 0x00 0x00
     1 byte           4 bytes          4 bytes
```

---

### 【第 2 步】Host 调用 polling_station.init()

**源代码（src/lib.rs）：**
```rust
pub fn init(&self) -> Result<InitMessage> {
    tracing::info!("init");
    let env = ExecutorEnv::builder().write(&self.state)?.build()?;
    let prover = default_prover();
    let receipt = prover.prove(env, INIT_ELF)?.receipt;
    Ok(InitMessage { receipt })
}
```

**执行过程：**
```
1. ExecutorEnv::builder().write(&self.state)?
   ↓
   构造一个"执行环境"对象
   将 VotingMachineState 的字节序列化后放进去
   
   ┌─────────────────────────────┐
   │  ExecutorEnv                │
   ├─────────────────────────────┤
   │ stdin: [0x01, 0x00, ...]    │  ← state 的字节
   │ stdout: <empty>             │
   │ (其他配置项)                │
   └─────────────────────────────┘

2. default_prover() 和 prove(...)
   ↓
   启动 RISC Zero ZKVM
   加载 INIT_ELF（编译好的 guest 程序二进制）
   执行 guest 程序（init.rs 中的 main()）
```

---

### 【第 3 步】Guest 开始执行（init.rs 的 main 函数）

**源代码（methods/guest/src/bin/init.rs）：**
```rust
fn main() {
    let state: VotingMachineState = env::read();
    // ...
}
```

**执行细节：**

```
↓ ZKVM 启动
├─ 加载 INIT_ELF 到虚拟机内存
├─ 从 stdin 读取 Host 写入的数据
│  ↓
│  stdin buffer: [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
│  
├─ 反序列化字节为 VotingMachineState
│  ↓
│  Deserialization 过程：
│    ├─ 读取第 1 字节 0x01 → polls_open = true
│    ├─ 读取第 2-5 字节 0x00000000 → voter_bitfield = 0
│    └─ 读取第 6-9 字节 0x00000000 → count = 0
│  ↓
└─ state 变量被创建

┌──────────────────────────────────┐
│  Guest 内存中的 state            │
├──────────────────────────────────┤
│  polls_open: true                │
│  voter_bitfield: 0               │
│  count: 0                        │
└──────────────────────────────────┘
```

---

### 【第 4 步】Guest 序列化状态为字节

**源代码：**
```rust
let state_bytes = to_vec(&state).unwrap();
```

**执行细节：**

```
state（Rust 对象）
    ↓ to_vec(&state)
    ↓ 通过 serde 进行序列化
    ↓ 遍历每个字段并转成字节
    ↓
state_bytes（Vec<u8>）

┌─────────────────────────────────────────┐
│  state_bytes = Vec<u8> {                │
│    capacity: 9,                         │
│    len: 9,                              │
│    ptr: 0x7fff1234 (指向堆上的数据)    │
│  }                                      │
│                                         │
│  堆上的数据：                           │
│  [0x01, 0x00, 0x00, 0x00,              │
│   0x00, 0x00, 0x00, 0x00, 0x00]       │
│   ↑     ↑           ↑        ↑         │
│   |     |           |        |         │
│   |  voter_bitfield | count  │         │
│   polls_open        (4 bytes) (4 bytes)│
│   (1 byte)                             │
└─────────────────────────────────────────┘
```

---

### 【第 5 步】Guest 计算状态哈希

**源代码：**
```rust
let state_hash = *Impl::hash_words(&state_bytes);
```

**执行细节：**

```
state_bytes = [0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    ↓
    通过 SHA256 算法计算
    ↓
    (这是一个复杂的密码学运算，涉及多轮哈希迭代)
    ↓
state_hash = Digest { 
    /* 32 字节的二进制数据 */
}

具体输出（示例，实际会不同）：
state_hash = 0xA3F5B2C1D8E4F7A9B6C2D5E8F1A4B7C0 (32 bytes = 256 bits)
           或用十六进制表示：
           [0xA3, 0xF5, 0xB2, 0xC1, 0xD8, 0xE4, 0xF7, 0xA9,
            0xB6, 0xC2, 0xD5, 0xE8, 0xF1, 0xA4, 0xB7, 0xC0,
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22,
            0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0x00]

⚠️  重要：
    • 同一个状态，每次哈希都得到相同的值
    • 改一点点状态（如 polls_open: false），哈希完全不同
    • 哈希是单向的：无法从哈希值反推原始状态
```

---

### 【第 6 步】Guest 创建 Commit 并提交到 Journal

**源代码：**
```rust
env::commit(&InitializeVotingMachineCommit {
    polls_open: state.polls_open,
    voter_bitfield: state.voter_bitfield,
    state: state_hash,
});
```

**执行细节：**

```
1. 创建 InitializeVotingMachineCommit 对象：
   
   ┌──────────────────────────────────────────┐
   │  InitializeVotingMachineCommit {         │
   ├──────────────────────────────────────────┤
   │  polls_open: true                        │  ← bool（1 字节）
   │  voter_bitfield: 0                       │  ← u32（4 字节）
   │  state: Digest {                         │
   │    /* SHA256 的 32 字节结果 */           │
   │  }                                       │
   └──────────────────────────────────────────┘
   
   总大小：1 + 4 + 32 = 37 字节

2. 序列化为字节：
   
   commit_bytes = [
       0x01,                        // polls_open = true
       0x00, 0x00, 0x00, 0x00,    // voter_bitfield = 0
       0xA3, 0xF5, 0xB2, 0xC1, .. // state hash（32 字节）
   ]

3. 提交到 Journal：
   
   env::commit(&commit_object)
       ↓
   ┌─────────────────────────────────────────────┐
   │  Journal（ZKVM 的日志）                     │
   ├─────────────────────────────────────────────┤
   │  [                                          │
   │    Entry 0: InitializeVotingMachineCommit {│
   │      polls_open: true,                      │
   │      voter_bitfield: 0,                     │
   │      state: Digest { /* 32 bytes */ }      │
   │    }                                        │
   │  ]                                          │
   └─────────────────────────────────────────────┘
```

---

### 【第 7 步】ZKVM 生成证明（Receipt）

**过程：**

```
ZKVM 已执行完 Guest 程序
    ↓
收集执行轨迹（Trace）：
    ├─ 每一步执行的寄存器状态
    ├─ 内存访问记录
    ├─ Journal 中的所有 commit
    └─ stdout/stdin 数据

    Trace 示例：
    ┌──────────────────────────────────────┐
    │ Step 0: PC=0x1000, reg_a=0, ...     │
    │ Step 1: PC=0x1004, reg_a=1, ...     │
    │ Step 2: read from stdin: 0x01       │
    │ Step 3: deserialize: polls_open=true│
    │ ...                                  │
    │ Step N: env::commit(...)            │
    │ Step N+1: VM halts normally         │
    └──────────────────────────────────────┘
    ↓
使用密码学技术证明这个执行过程：
    ├─ 使用 RISC Zero 的证明系统
    │  （基于 STARK / Groth16）
    ├─ 压缩执行轨迹为简洁证明
    └─ 证明是自包含的，无需重新执行
    ↓
生成最终 Receipt：

┌────────────────────────────────────────────────┐
│  Receipt {                                     │
│    proof: <密码学证明>,                        │
│    journal: [                                  │
│      InitializeVotingMachineCommit {          │
│        polls_open: true,                       │
│        voter_bitfield: 0,                      │
│        state: Digest { /* 32 bytes */ }       │
│      }                                         │
│    ]                                          │
│  }                                            │
└────────────────────────────────────────────────┘
```

---

### 【第 8 步】Host 接收 Receipt

**源代码（src/lib.rs）：**
```rust
let receipt = prover.prove(env, INIT_ELF)?.receipt;
Ok(InitMessage { receipt })
```

**执行细节：**

```
ZKVM 返回执行结果：
    ↓
┌────────────────────────────────────────────────┐
│  Receipt {                                     │
│    proof: <4-8 KB 的密码学证明>,                │
│    journal: [                                  │
│      InitializeVotingMachineCommit { ... }    │
│    ]                                          │
│  }                                            │
└────────────────────────────────────────────────┘
    ↓
包装为 InitMessage：
    ↓
┌────────────────────────────────────────────────┐
│  InitMessage {                                 │
│    receipt: Receipt { ... }                    │
│  }                                            │
└────────────────────────────────────────────────┘
    ↓
返回给 Host 程序
```

---

### 【第 9 步】验证者验证 Receipt

**代码（tests 中）：**
```rust
let init_msg = polling_station.init().unwrap();
let init_state = init_msg.verify_and_get_commit()?;
```

**执行细节：**

```
验证过程：
    ↓
1. 验证证明的真实性：
   
   receipt.verify(INIT_ID)?
       ↓
   使用 INIT_ID（Guest 程序的哈希值）
   验证证明确实是由这个程序生成的
       ↓
   验证证明是否有效（密码学验证）
       ↓
   如果验证失败：返回错误
   如果验证成功：继续

2. 从 Journal 中提取 Commit：
   
   journal.decode()?
       ↓
   从 Journal 的字节流中反序列化
       ↓
   ┌────────────────────────────────────┐
   │  InitializeVotingMachineCommit {  │
   │    polls_open: true,               │
   │    voter_bitfield: 0,              │
   │    state: Digest { /* ... */ }    │
   │  }                                │
   └────────────────────────────────────┘
       ↓
   返回给验证者

3. 验证者现在知道：
   
   ✓ 初始状态确实是：
     - polls_open = true
     - voter_bitfield = 0
     - 状态的哈希值是确定的
   
   ✓ 这是由 INIT_ID 程序生成的证明
   
   ✓ 证明无法伪造（密码学保证）
   
   ✓ 但验证者不知道：
     - 完整的执行轨迹
     - ZKVM 内部的所有步骤
     - （这就是"零知识"的体现）
```

---

## 数据大小参考

| 项目 | 大小 | 说明 |
|------|------|------|
| `VotingMachineState` | 9 字节 | 2 bool + 2 u32 |
| `Digest` | 32 字节 | SHA256 哈希 |
| `InitializeVotingMachineCommit` | 37 字节 | bool + u32 + Digest |
| `Receipt` | 5-10 KB | 包含证明 + Journal |
| Guest ELF 文件 | ~1 MB | 编译后的二进制 |

---

## 关键概念回顾（数据角度）

### 1. 为什么需要哈希（state_hash）？

```
问题：Host 说"我发送了某个状态"，但如何证明？

直接方法（不安全）：
  Host 保存完整状态 VotingMachineState { ... }
  问题：
    - 状态可能很大
    - Host 可能伪造或篡改
    - Guest 无法直接验证

零知识证明方法（安全）：
  Guest 计算状态的哈希
  哈希放进 Journal 中
  问题解决：
    - 哈希大小固定（32 字节）
    - 哈希是单向的（无法反演）
    - Host 无法伪造相同的哈希
    - 任何人都能验证一个状态是否对应这个哈希
```

### 2. 为什么需要 env::commit？

```
Journal = ZKVM 内部的"公告板"

如果没有 env::commit：
  ├─ Guest 执行完毕
  ├─ Host 收到 Receipt
  └─ 但 Host 不知道 Guest 执行了什么
     （Guest 可能只是闲置，没有工作）

有了 env::commit：
  ├─ Guest 执行完毕
  ├─ env::commit(&commit_data) 记录关键信息
  ├─ Journal 包含这个信息
  ├─ Receipt 包含 Journal
  ├─ Host 可以读取 Journal
  └─ Host 确认 Guest 确实执行了预期的逻辑
```

### 3. 为什么需要 verify？

```
问题：Receipt 可能被伪造或篡改

解决：
  receipt.verify(INIT_ID)?
    ↓
  使用密码学验证
  确认：
    - 这个 Receipt 是由 INIT_ID 程序生成的
    - Receipt 没有被篡改
    - 如果验证失败，立即返回错误
    
  这样，验证者可以确信 Journal 中的数据是真实的
```

---

## 完整数据流总结

```
时间线：

[Host]                          [Guest (ZKVM)]               [Journal]
  |                                  |                           |
  | 1. 创建状态                      |                           |
  | state = { true, 0, 0 }          |                           |
  |                                 |                           |
  | 2. 调用 init()                   |                           |
  |--------(state)-----------→      |                           |
  |     env::write(&state)  |      |                           |
  |                         |      | 3. env::read()            |
  |                         |      | state = { true, 0, 0 }    |
  |                         |      |                           |
  |                         |      | 4. 序列化                 |
  |                         |      | state_bytes = [0x01, ...] |
  |                         |      |                           |
  |                         |      | 5. 计算哈希               |
  |                         |      | state_hash = Digest{...}  |
  |                         |      |                           |
  |                         |      | 6. 创建 Commit            |
  |                         |      | commit = Commit {         |
  |                         |      |   polls_open: true,       |
  |                         |      |   voter_bitfield: 0,      |
  |                         |      |   state: Digest{...}      |
  |                         |      | }                         |
  |                         |      |                           |
  |                         |      | 7. env::commit()          |
  |                         |      |-----(commit)----→         |
  |                         |      |                      Entry[0] = Commit
  |                         |      |                           |
  | 8. 证明生成完毕        |      |                           |
  | Receipt {             |      |                           |
  |   proof: ...,         |      |                           |
  |   journal: [Commit]   |←---(包含 Journal)----|         |
  | }                     |      |                           |
  |                       |      |                           |
  | 9. verify(&Receipt)   |      |                           |
  | 密码学验证成功 ✓       |      |                           |
  |                       |      |                           |
  | 10. journal.decode()  |      |                           |
  | 提取 Commit ✓        |      |                           |
```

---

## 常见问题与答案

### Q1: 为什么 state 要序列化为字节？

**A:** 因为哈希函数（SHA256）接收字节数组作为输入。Rust 的结构体是内存对象，不能直接哈希。序列化过程将其转为标准的字节格式，这样同一个结构体总是哈希到相同的值。

### Q2: `*Impl::hash_words(&state_bytes)` 中的 `*` 是什么？

**A:** `Impl::hash_words()` 返回引用 `&Digest`，`*` 解引用操作符取出实际的 `Digest` 值。如果不解引用，`state_hash` 会是引用，类型不匹配。

### Q3: Guest 中为什么不用 println! 调试？

**A:** Guest 是 `#![no_std]` 的（不使用标准库），没有 `println!`。可以用 `env::write()` 向 stdout 写数据，或通过 `env::commit()` 记录调试信息。

### Q4: Journal 中的数据是否私密？

**A:** 否。Journal 被包含在 Receipt 中，任何人都能读取。所以 Commit 中不能包含敏感信息（如密码）。这就是为什么只记录哈希和状态标志，而不记录完整的投票内容。

### Q5: 多次哈希同一个状态，结果相同吗？

**A:** 是的。SHA256 是确定性函数，同一个输入总是产生相同的输出。这保证了数据的一致性。

---

## 下一步：理解 SubmitBallotCommit

类似的流程也存在于 `submit.rs` 中，但增加了：
- 输入参数（Ballot）
- 状态转换（旧状态 → 新状态）
- 两个哈希值（old_state, new_state）

这部分内容可以参考 `LEARNING_GUIDE_SubmitBallot.md`（待创建）。
