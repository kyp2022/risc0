# 投票机示例：详细学习指南（附Rust语法讲解）

## 目录
1. [测试代码逐行讲解](#测试代码逐行讲解)
2. [Rust基础语法速成](#rust基础语法速成)
3. [执行流程动画式讲解](#执行流程动画式讲解)
4. [关键概念深入](#关键概念深入)
5. [常见问题解答](#常见问题解答)

---

## 测试代码逐行讲解

### 测试函数开头（`src/lib.rs` 中的 `tests` 模块）

```rust
#[cfg(test)]
mod tests {
    use test_log::test;
    use super::*;

    #[test]
    fn protocol() {
        // ... 测试代码 ...
    }
}
```

#### Rust语法详解

**`#[cfg(test)]`** — 属性（Attribute）
- 含义：告诉编译器"这个模块只在运行 `cargo test` 时编译，正常 `cargo build` 时忽略"
- 作用：避免生产二进制中包含测试代码，减小文件大小
- 类比：C/C++ 的 `#ifdef TEST ... #endif`

**`mod tests { ... }`** — 模块（Module）
- 含义：在当前文件中声明一个作用域/命名空间，防止命名冲突
- 内部定义的函数、结构体只在模块内可见（除非标记 `pub`）
- 类比：C++ 的命名空间 `namespace tests { ... }`

**`use test_log::test;`** — 导入（Use）
- 含义：从外部crate `test_log` 导入 `test` 宏
- 作用：替换标准 `#[test]` 宏，增加日志记录功能（方便调试）
- 语法：`use 包名::模块名::项名;`

**`use super::*;`** — 导入父模块的所有公开项
- 含义：`super` = 父模块，`*` = 所有公开内容
- 作用：让测试可以访问 `PollingStation`、`VotingMachineState` 等类型
- 等价形式：`use crate::*;` 或具体导入 `use crate::{PollingStation, Ballot};`

**`#[test]`** — 测试属性
- 含义：标记下面的函数为测试用例（`cargo test` 会执行所有 `#[test]` 函数）
- 要求：函数不能有参数，无返回值（或返回 `Result<(), E>`）

---

### 测试函数主体（第1部分：初始化状态）

```rust
#[test]
fn protocol() {
    // 第1部分：创建初始状态
    let polling_station_state = VotingMachineState {
        polls_open: true,
        voter_bitfield: 0,
        count: 0,
    };

    let mut polling_station = PollingStation::new(polling_station_state);
```

#### Rust语法详解

**`let` 和 `mut` 关键字**

```rust
let polling_station_state = VotingMachineState { ... };
```

- **`let`**：声明变量（Rust 的 `var` / `const`）
- **默认不可变**：与 C/JavaScript 不同，Rust 变量默认是 **只读** 的（immutable）
- **可以重影（shadowing）**：即使已声明也可以 `let x = new_value;` 重新声明

```rust
let mut polling_station = PollingStation::new(...);
```

- **`mut`**：声明为可变（mutable）— 之后可以修改
- **后面会用到**：`polling_station.submit(...)` 方法需要修改内部状态 `self.state`

**结构体字面量语法**

```rust
VotingMachineState {
    polls_open: true,
    voter_bitfield: 0,
    count: 0,
}
```

- **`{ field: value, ... }`**：Rust 的结构体初始化语法（C 类比：`{.field = value, ...}`）
- **必须指定所有字段**：除非结构体有默认实现（`Default` trait）
- **类型自动推导**：编译器从上下文推断 `polling_station_state` 的类型为 `VotingMachineState`

**`::` 操作符**

```rust
PollingStation::new(polling_station_state)
```

- **`::`**：访问关联项（associated items）— 通常用于静态方法或关联函数
- **对比 `.`**：`.` 用于实例方法（需要 `self` 参数），`::` 用于静态方法（无需实例）
- **类比**：C++ 的 `ClassName::staticMethod()` vs `obj.method()`

---

### 第2部分：创建测试选票

```rust
    // 第2部分：创建选票
    let ballot1 = Ballot {
        voter: 0,
        vote_yes: false,  // 投票内容：投"否"票
    };
    let ballot2 = Ballot {
        voter: 1,
        vote_yes: true,   // 投"是"票
    };
    let ballot3 = Ballot {
        voter: 2,
        vote_yes: true,
    };
    let ballot4 = Ballot {
        voter: 1,         // 同一投票者的重复投票
        vote_yes: false,
    };
    let ballot5 = Ballot {
        voter: 3,
        vote_yes: false,
    };
    let ballot6 = Ballot {
        voter: 4,
        vote_yes: true,   // 在投票站关闭后的投票（会失败）
    };
```

#### Rust语法详解

**结构体字面量（重复多次）**

- 同 `VotingMachineState` 的初始化方式
- 注意：这里创建的 `ballot1-6` 都是不可变的（没有 `mut`）
- 因为我们只在创建后传递给 `submit()`，不再修改

**Rust 的所有权（Ownership）简介**

这很重要！当你写：
```rust
let ballot1 = Ballot { voter: 0, vote_yes: false };
polling_station.submit(&ballot1)
```

- **`ballot1`** 的所有权属于 `main` 函数
- **`&ballot1`**（取引用）：允许 `submit()` 借用（borrow）`ballot1`，但不转移所有权
- **之后仍可使用 `ballot1`**：因为只是借用，不是转移
- **如果没有 `&`，会报错**："value used after move"

---

### 第3部分：执行投票流程（核心逻辑）

```rust
    // 第3部分：执行投票流程
    let init_msg = polling_station.init().unwrap();
    let ballot_msg1 = polling_station.submit(&ballot1).unwrap();
    let ballot_msg2 = polling_station.submit(&ballot2).unwrap();
    let ballot_msg3 = polling_station.submit(&ballot3).unwrap();
    let ballot_msg4 = polling_station.submit(&ballot4).unwrap();
    let ballot_msg5 = polling_station.submit(&ballot5).unwrap();
    let close_msg = polling_station.freeze().unwrap();
    let ballot_msg6 = polling_station.submit(&ballot6).unwrap();
```

#### Rust语法详解

**方法调用与 `self` 参数**

```rust
let init_msg = polling_station.init().unwrap();
```

- **`.init()`**：实例方法（receiver 是 `&self`，表示不可变借用）
  - 对应定义：`pub fn init(&self) -> Result<InitMessage> { ... }`
  - 只读取 `self.state`，不修改
  
- **`.submit(&ballot1)`**：实例方法（receiver 是 `&mut self`，表示可变借用）
  - 对应定义：`pub fn submit(&mut self, ballot: &Ballot) -> Result<SubmitBallotMessage> { ... }`
  - 需要修改 `self.state`，所以 `polling_station` 必须是 `mut`

- **`.freeze()`**：实例方法（receiver 是 `&mut self`）
  - 类似 `submit()`，会修改状态

**`Result<T, E>` 与 `.unwrap()`**

```rust
let init_msg = polling_station.init().unwrap();
```

`init()` 的返回类型是 `Result<InitMessage>`（等同于 `Result<InitMessage, Error>`）

- **`Result<T, E>`**：Rust 的错误处理类型，要么是 `Ok(T)`，要么是 `Err(E)`
- **`.unwrap()`**：提取 `Ok` 中的值，如果是 `Err` 则 panic（程序崩溃）
- **用途**：在测试中快速处理错误，无需详细的错误处理

**等价的详细写法（更安全）**：
```rust
let init_msg = match polling_station.init() {
    Ok(msg) => msg,
    Err(e) => panic!("init failed: {}", e),
};
```

**Rust 错误处理哲学**
- 与 JavaScript 的 `try/catch` 或 Java 的 `throws` 不同
- **编译时强制处理**：必须明确处理 `Err` 分支，否则编译失败
- **无隐藏异常**：调用者明确知道函数可能失败

---

### 第4部分：验证最终结果

```rust
    // 第4部分：断言验证
    assert_eq!(polling_station.state.count, 2);
```

#### Rust语法详解

**访问结构体字段**

```rust
polling_station.state.count
```

- **`.state`**：访问 `polling_station` 的 `state` 字段（类型 `VotingMachineState`）
- **`.count`**：再访问 `state` 的 `count` 字段（类型 `u32`）
- **链式访问**：类似 JavaScript 的 `obj.field1.field2`

**宏（Macro）— `assert_eq!()`**

```rust
assert_eq!(polling_station.state.count, 2);
```

- **`assert_eq!(left, right)`**：如果 `left != right`，则 panic 并打印详细信息
- **宏 vs 函数**：`!` 后缀表示是宏（可变参数、编译时代码生成）
- **其他常用宏**：`assert!()`, `panic!()`, `println!()`, `format!()`

---

### 第5部分：验证证明（Receipt）

```rust
    // 第5部分：验证所有证明
    let init_state = init_msg.verify_and_get_commit();
    let ballot_commit1 = ballot_msg1.verify_and_get_commit();
    let ballot_commit2 = ballot_msg2.verify_and_get_commit();
    // ... 更多验证 ...
    let ballot_commit6 = ballot_msg6.verify_and_get_commit();
```

#### Rust语法详解

**方法链（Method chaining）与返回值**

```rust
let init_state = init_msg.verify_and_get_commit();
```

- **`verify_and_get_commit()`**：方法定义中返回 `Result<InitializeVotingMachineCommit>`
- **这里省略了 `.unwrap()`**，所以 `init_state` 的类型是 `Result<...>`
- **后面的代码继续调用这些结果而不检查**：有点不安全，但在测试中可以接受

实际方法实现（来自 `src/lib.rs`）：
```rust
impl InitMessage {
    pub fn verify_and_get_commit(&self) -> Result<InitializeVotingMachineCommit> {
        self.receipt.verify(INIT_ID)?;
        self.get_state()
    }
}
```

**`?` 操作符（错误传播）**

```rust
self.receipt.verify(INIT_ID)?;
```

- **`?` = unwrap + 传播**：如果是 `Err`，立即返回给调用者；如果是 `Ok`，提取值继续执行
- **只能在返回 `Result` 的函数中使用**
- **等价的详细写法**：
  ```rust
  match self.receipt.verify(INIT_ID) {
      Ok(v) => v,
      Err(e) => return Err(e),
  }
  ```

---

### 第6部分：日志输出

```rust
    tracing::info!("initial commit: {:?}", init_state);
    tracing::info!("ballot 1: {:?}", ballot1);
    tracing::info!("ballot 1 commit: {:?}", ballot_commit1);
    // ... 更多日志 ...
    tracing::info!("Final vote count: {:?}", polling_station.state.count);
```

#### Rust语法详解

**日志宏 `tracing::info!()`**

```rust
tracing::info!("initial commit: {:?}", init_state);
```

- **`tracing::info!(...)`**：日志库的信息级别（来自 `tracing` crate）
- **`"..."` — 格式字符串**：使用 `{:?}` 作为占位符（调试格式）
- **`{:?}` 与 `{}` 的区别**：
  - `{}` = Display（用户友好的格式，需要实现 Display trait）
  - `{:?}` = Debug（开发者友好的格式，自动派生）
  
例如：
```rust
let x = 42;
println!("{}", x);   // 输出：42
println!("{:?}", x); // 输出：42

#[derive(Debug)]
struct Point { x: i32, y: i32 }
let p = Point { x: 1, y: 2 };
println!("{:?}", p); // 输出：Point { x: 1, y: 2 }
// println!("{}", p);  // 错误！Point 没有实现 Display
```

**运行测试时显示日志**

```bash
RUST_LOG=info cargo test --release -- --nocapture
```

- **`RUST_LOG=info`**：环境变量，设置日志级别为 info（还有 warn, debug, trace 等）
- **`--nocapture`**：cargo test 通常隐藏 stdout，这个标志显示所有输出

---

## Rust基础语法速成

### 基本类型

```rust
let a: u32 = 42;         // unsigned 32-bit integer（0 到 4,294,967,295）
let b: i32 = -10;        // signed 32-bit integer
let c: bool = true;      // boolean
let d: &str = "hello";   // string slice（不可变）
let mut s = String::from("world"); // String（可变、可扩展）
s.push_str("!");         // 修改字符串
```

### 所有权与借用

**所有权规则**：
1. 每个值在 Rust 中都有唯一的所有者
2. 值可以被转移（move）给新的所有者
3. 当所有者被 drop，值也被回收（自动释放内存）

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 的所有权转移给 s2，之后 s1 不能用
// println!("{}", s1);  // 编译错误

let s3 = String::from("world");
let s4 = &s3;  // s4 借用 s3（不转移所有权）
println!("{}", s3);  // 正常，s3 仍然可用
```

**借用分类**：
- **不可变借用 `&T`**：多个可以同时存在，只读
- **可变借用 `&mut T`**：同时只能有一个，可读写

```rust
let mut x = 10;
let r1 = &x;     // 不可变借用
let r2 = &x;     // 可以有多个
// let r3 = &mut x;  // 错误！不能混合可变和不可变借用
println!("{}, {}", r1, r2);  // 正常

let y = &mut x;  // 可变借用
*y += 1;         // 修改 x（通过 y）
println!("{}", x);  // 11
```

### 结构体（Struct）

```rust
#[derive(Debug, Clone)]  // 自动派生 Debug 和 Clone trait
struct VotingMachineState {
    polls_open: bool,
    voter_bitfield: u32,
    count: u32,
}

// 创建实例
let mut state = VotingMachineState {
    polls_open: true,
    voter_bitfield: 0,
    count: 0,
};

// 访问字段
println!("{}", state.count);  // 0

// 修改字段
state.count += 1;
println!("{}", state.count);  // 1
```

### 实现方法（impl 块）

```rust
impl VotingMachineState {
    // 关联函数（静态方法）— 无 self
    pub fn new(polls_open: bool) -> Self {
        VotingMachineState {
            polls_open,
            voter_bitfield: 0,
            count: 0,
        }
    }
    
    // 方法 — 有 &self（不可变）
    pub fn is_open(&self) -> bool {
        self.polls_open
    }
    
    // 方法 — 有 &mut self（可变）
    pub fn vote(&mut self, voter: u32, vote_yes: bool) -> bool {
        let voter_mask = 1 << voter;
        if self.polls_open && 0 == self.voter_bitfield & voter_mask {
            self.voter_bitfield |= voter_mask;
            if vote_yes {
                self.count += 1;
            }
            true
        } else {
            false
        }
    }
}

// 使用
let mut state = VotingMachineState::new(true);
println!("{}", state.is_open());  // true
let voted = state.vote(0, true);
println!("{}", voted);  // true
println!("{}", state.count);  // 1
```

### 模式匹配 （match）

```rust
let result: Result<i32, &str> = Ok(42);

match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => println!("Error: {}", e),
}

// 简化版本
if let Ok(value) = result {
    println!("Success: {}", value);
}
```

### 特征（Trait）— 类似接口

```rust
trait Hashable {
    fn compute_hash(&self) -> u64;
}

struct Data(u32);

impl Hashable for Data {
    fn compute_hash(&self) -> u64 {
        self.0 as u64 * 31
    }
}

let d = Data(10);
println!("{}", d.compute_hash());  // 310
```

---

## 执行流程动画式讲解

### 时间线视图

```
时间 ⬇

[1] 创建初始状态 PollingStation
    state = { polls_open: true, voter_bitfield: 0, count: 0 }

[2] 调用 init()
    ➜ Host: 将 state 写入 ExecutorEnv（输入）
    ➜ Guest (init.rs): 读取 state，计算 SHA256 哈希
    ➜ Guest: env::commit(InitializeVotingMachineCommit { ... })
    ➜ Host: 接收 Receipt，包含 journal 和证明
    ⬅ 返回 InitMessage { receipt }

[3] 调用 submit(&ballot1: voter=0, vote_yes=false)
    ➜ Host: 创建 SubmitBallotParams { state, ballot }
    ➜ Host: 将 params 写入 ExecutorEnv
    ➜ Guest (submit.rs): 
        1. 读取 params
        2. 调用 params.process()
           - 在 core 中：state.vote(0, false)
             * 检查 polls_open? YES
             * 检查 bit[0] 未设置? YES
             * 设置 bit[0] = 1（voter_bitfield = 0b1）
             * vote_yes==false，所以 count 不增加
             * 返回 true
        3. 新 state = { polls_open: true, voter_bitfield: 1, count: 0 }
        4. 计算旧状态哈希和新状态哈希
        5. 将新 state 写入 stdout（env::write()）
        6. env::commit(SubmitBallotCommit { old_state, new_state, vote_counted: true, ... })
    ➜ Host: 从输出读取新 state，更新 self.state
    ⬅ 返回 SubmitBallotMessage { receipt }
    
    [Host 的 self.state 现在更新为] { polls_open: true, voter_bitfield: 1, count: 0 }

[4] 调用 submit(&ballot2: voter=1, vote_yes=true)
    ➜ 同上流程，但：
        - state.vote(1, true)
        - 设置 bit[1] = 1（voter_bitfield = 0b11）
        - vote_yes==true，所以 count += 1
        - 新 state = { polls_open: true, voter_bitfield: 3, count: 1 }
    
    [Host 的 self.state 现在] { polls_open: true, voter_bitfield: 3, count: 1 }

[5] 调用 submit(&ballot3: voter=2, vote_yes=true)
    ➜ state.vote(2, true)
    ➜ 新 state = { polls_open: true, voter_bitfield: 7, count: 2 }

[6] 调用 submit(&ballot4: voter=1, vote_yes=false)
    ➜ state.vote(1, false)
    ➜ 检查 polls_open? YES
    ➜ 检查 bit[1] 未设置? NO（已经在 [4] 中设置）
    ➜ 返回 false（vote_counted=false）
    ➜ 状态不变：{ polls_open: true, voter_bitfield: 7, count: 2 }

[7] 调用 submit(&ballot5: voter=3, vote_yes=false)
    ➜ state.vote(3, false)
    ➜ 返回 true
    ➜ 新 state = { polls_open: true, voter_bitfield: 15, count: 2 }

[8] 调用 freeze()
    ➜ Host: 创建 FreezeVotingMachineParams { state }
    ➜ Guest (freeze.rs):
        1. 读取 params
        2. state.polls_open = false
        3. 新 state = { polls_open: false, voter_bitfield: 15, count: 2 }
        4. 将新 state 写回 stdout
        5. env::commit(FreezeVotingMachineCommit { count: 2, ... })
    ➜ Host: 更新 self.state
    ⬅ 返回 FreezeStationMessage { receipt }

[9] 调用 submit(&ballot6: voter=4, vote_yes=true)
    ➜ state.vote(4, true)
    ➜ 检查 polls_open? NO（已在 [8] 冻结）
    ➜ 返回 false（vote_counted=false）
    ➜ 状态不变：{ polls_open: false, voter_bitfield: 15, count: 2 }

[10] 最终验证
     ➜ assert_eq!(polling_station.state.count, 2)  ✓ PASS
```

---

## 关键概念深入

### 1. 零知识证明（ZKP）中的 Guest vs Host

**Guest（隔离的 RISC-V 虚拟机）**：
- 运行受限的 Rust 代码（`#![no_std]`，无标准库）
- **不能**直接访问 Host 的文件系统、网络、时间等
- **只能**通过 `env::read()` 接收 Host 输入
- **只能**通过 `env::write(...)` 和 `env::commit(...)` 输出
- 所有操作被记录和证明（proof）

**Host（标准的 Rust 程序）**：
- 有完整的标准库和外界访问
- 负责构造输入、调用 Prover、处理证明
- 从 Guest 的证明中提取 `journal`（public outputs）
- 可以验证证明的有效性（`receipt.verify()`）

**数据流示意**：
```
Host                 Ghost (Prover)              Blockchain/Verifier
┌─────────┐         ┌──────────────────┐        ┌────────────────┐
│ 创建    │────────→│ 执行 Guest 代码  │───────→│ 输出：Receipt  │
│ 输入    │ write   │ 计算状态转换     │ commit │ - Journal      │
│         │         │ 计算哈希/证明    │        │ - Proof        │
└─────────┘         └──────────────────┘        └────────────────┘
   ↑                                                     ↓
   └──────────────── Host 读取输出 ─────────────────────┘
                    from_slice(&output)
                    receipt.verify()
```

### 2. 位操作（Bitfield）详解

投票机使用 `u32` 的 32 位来追踪 32 个投票者是否已投票：

```rust
let mut voter_bitfield: u32 = 0;  // 初始：0b0000_0000...（全0）

// 投票者0投票
let voter = 0;
let voter_mask = 1 << voter;      // 1 << 0 = 0b0001
voter_bitfield |= voter_mask;     // 按位或：0 | 1 = 1，所以 voter_bitfield = 1

// 投票者2投票
let voter = 2;
let voter_mask = 1 << voter;      // 1 << 2 = 0b0100 = 4
voter_bitfield |= voter_mask;     // 1 | 4 = 5 = 0b0101

// 检查投票者1是否已投票
let voter = 1;
let voter_mask = 1 << voter;      // 1 << 1 = 0b0010 = 2
if 0 == voter_bitfield & voter_mask {  // 5 & 2 = 0b0101 & 0b0010 = 0b0000 = 0
    println!("投票者1还未投票");
} else {
    println!("投票者1已投票");
}
```

**位操作速查表**：
```rust
a << n       // 左移 n 位（乘以 2^n）
a >> n       // 右移 n 位（除以 2^n）
a | b        // 按位或（设置位）
a & b        // 按位与（检查位）
a ^ b        // 按位异或（翻转位）
!a           // 按位非（反转）
```

### 3. 序列化与哈希

**序列化（Serde）**：
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Data { x: i32 }

let data = Data { x: 42 };
let bytes = bincode::serialize(&data).unwrap();  // 转换为字节数组
let restored: Data = bincode::deserialize(&bytes).unwrap();  // 从字节恢复
```

**在示例中**：
```rust
let state_bytes = to_vec(&state).unwrap();  // 将状态序列化为 Vec<u8>
let state_hash = *Impl::hash_words(&state_bytes);  // SHA256 哈希
```

**哈希的作用**：
- 用于证明完整性（状态未被篡改）
- 在不公开完整状态的情况下验证正确性
- 类比：数字指纹

---

## 常见问题解答

### Q1：为什么 Guest 不能使用标准库？

**A**：因为 Guest 在隔离的虚拟机中运行（RISC-V），不能访问操作系统功能。如果允许使用标准库，Guest 可能会执行非确定性操作（如读取随机数），导致证明失效。

### Q2：`voter_bitfield` 用 u32 为什么最多支持 32 个投票者？

**A**：u32 = 32 bit，每个 bit 代表一个投票者。若要支持更多投票者，可用：
- u64（64 个）
- u128（128 个）
- Vec<u64> 或 BitVec（任意多）

但这会增加哈希计算成本和证明大小。

### Q3：为什么要分别保存 `old_state` 和 `new_state` 的哈希？

**A**：
1. **证明状态转换的正确性**：验证者可以检查新状态确实是从旧状态按规则得出
2. **防止篡改**：如果有人修改了 commit 中的 count，哈希就不匹配
3. **区块链应用**：类似交易链，每个操作都链接到前一个状态

### Q4：`&self` vs `&mut self` 如何选择？

**A**：
- **`&self`**：方法只读取状态，不修改。例如 `get_count()`
- **`&mut self`**：方法需要修改状态。例如 `vote()` 或 `submit()`
- **`self`**：方法消费所有权，调用后原变量不可用。例如 `into_string()`

### Q5：如何调试 Guest 代码？

**A**：
- **加日志**：在 Guest 中通过 `env::write()` 或 `env::commit()` 输出调试信息
- **单元测试**：为 Core 的业务逻辑编写纯 Rust 测试（无 Prover）
- **简化 Guest**：注释掉部分代码，逐步恢复找到问题

### Q6：为什么测试中有 `.unwrap()` 但不显示错误？

**A**：
```rust
let result = polling_station.init().unwrap();
```

- 如果初始化失败，`.unwrap()` 会 panic，测试停止并显示错误栈
- 在测试中这是可以接受的（快速反馈）
- 在生产代码中应该用 `?` 或 `match` 显式处理

---

## 总结与下一步

### 你现在理解了：
✓ Rust 的所有权与借用系统  
✓ 结构体、方法、trait  
✓ 结果处理（`Result<T, E>`）  
✓ Guest 和 Host 的通信  
✓ 零知识证明的工作流  
✓ 投票机示例的完整执行流程  

### 建议的下一步学习：
1. **修改示例**：
   - 把 `voter_bitfield` 改为 `Vec<bool>` 支持任意多投票者
   - 增加投票理由等元数据
   - 添加时间戳验证

2. **深入 RISC Zero**：
   - 学习如何编写自定义 Guest 程序
   - 理解 Proof 与 Receipt 的结构
   - 在链上验证证明

3. **优化**：
   - 减少证明大小
   - 提高验证速度
   - 支持并发投票

4. **扩展应用**：
   - 构建实际的投票 DApp
   - 整合区块链后端
   - 实现去中心化投票系统
