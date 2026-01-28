# Rust 语法速查（针对投票机示例）

用于快速理解本示例中出现的所有 Rust 语法特性。

---

## 1. 基础语法

### 变量声明与可变性

```rust
// 不可变变量（默认）
let x = 10;              // 自动推导类型为 i32
let x: u32 = 10;         // 显式指定类型为 u32
// x = 20;               // 错误！x 是不可变的

// 可变变量
let mut y = 10;
y = 20;                  // ✓ 正确

// 常量（编译时已知，不能改变）
const VOTER_COUNT: u32 = 32;

// 变量遮蔽（shadowing）— 用 let 重新声明
let x = 10;
let x = x + 1;           // ✓ 正确，x 现在是 11
```

### 类型系统

```rust
// 整数类型
u8, u16, u32, u64, u128    // 无符号（0 及以上）
i8, i16, i32, i64, i128    // 有符号（包括负数）

// 浮点
f32, f64

// 布尔
bool                        // true 或 false

// 字符
char                        // Unicode 字符，占 4 字节

// 字符串
"string"                    // &str — 不可变字符串切片（栈）
String::from("string")      // String — 可变动态字符串（堆）

// 数组（固定大小）
let arr: [i32; 3] = [1, 2, 3];
arr[0]                      // 访问元素

// 向量（动态大小）
let mut vec: Vec<i32> = vec![1, 2, 3];
vec.push(4);                // 添加元素
```

---

## 2. 结构体（Struct）

### 定义与创建

```rust
// 定义结构体
#[derive(Debug, Clone)]     // 属性 — 自动实现 Debug 和 Clone trait
struct VotingMachineState {
    polls_open: bool,       // 字段及其类型
    voter_bitfield: u32,
    count: u32,
}

// 创建实例
let state = VotingMachineState {
    polls_open: true,
    voter_bitfield: 0,
    count: 0,
};

// 访问字段
println!("{}", state.count);  // 0

// 修改字段（需要 mut）
let mut state = VotingMachineState { /* ... */ };
state.count = 10;

// 字段简写（当变量名等于字段名）
let polls_open = true;
let voter_bitfield = 0;
let count = 0;
let state = VotingMachineState {
    polls_open,       // 等价于 polls_open: polls_open,
    voter_bitfield,
    count,
};
```

### 元组结构体（Tuple Struct）

```rust
struct Point(i32, i32);
let p = Point(1, 2);
println!("{}", p.0);  // 1
```

### 实现方法

```rust
impl VotingMachineState {
    // 关联函数（静态方法）— 用 :: 调用
    pub fn new() -> Self {
        VotingMachineState {
            polls_open: true,
            voter_bitfield: 0,
            count: 0,
        }
    }
    
    // 方法 — 第一个参数是 self（不可变借用）
    pub fn is_open(&self) -> bool {
        self.polls_open
    }
    
    // 方法 — 可变借用 &mut self
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
    
    // 方法 — 消费所有权（self，无引用）
    pub fn into_string(self) -> String {
        format!("State: open={}", self.polls_open)
    }
}

// 调用
let mut state = VotingMachineState::new();
state.vote(0, true);
println!("{}", state.is_open());
let s = state.into_string();  // 之后 state 不可用
```

---

## 3. 所有权与借用

### 所有权规则

```rust
// 规则 1：每个值有唯一所有者
let s1 = String::from("hello");
let s2 = s1;  // 所有权转移给 s2
// println!("{}", s1);  // 错误！s1 失效

// 规则 2：函数接收所有权
fn take_string(s: String) {
    // s 在这里有所有权
    println!("{}", s);
}  // s 超出作用域，被释放

let s = String::from("hello");
take_string(s);
// println!("{}", s);  // 错误！所有权已转移

// 规则 3：函数返回所有权
fn create_string() -> String {
    String::from("world")
}
let s = create_string();  // 接收所有权

// 规则 4：Clone（显式复制）
let s1 = String::from("hello");
let s2 = s1.clone();  // 创建深拷贝
println!("{}", s1);   // ✓ s1 仍可用
println!("{}", s2);
```

### 借用（Borrow）

```rust
// 不可变借用 &T — 多个可同时存在
let s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{}, {}", r1, r2);  // ✓ 正确
// println!("{}", s);  // 仍可用

// 可变借用 &mut T — 同时只能有一个
let mut s = String::from("hello");
let r = &mut s;
r.push_str(" world");
println!("{}", r);   // "hello world"
println!("{}", s);   // "hello world"

// 混合错误
let mut s = String::from("hello");
let r1 = &s;        // 不可变借用
let r2 = &s;
// let r3 = &mut s;  // 错误！不能混合
println!("{}, {}", r1, r2);

// 函数中的借用
fn print_string(s: &String) {           // 借用，不转移所有权
    println!("{}", s);
}

fn modify_string(s: &mut String) {      // 可变借用
    s.push_str("!");
}

let mut s = String::from("hello");
print_string(&s);     // 借用
modify_string(&mut s); // 可变借用
println!("{}", s);     // "hello!"
```

### 引用 vs 解引用

```rust
let x = 5;
let r = &x;           // 引用：创建指向 x 的指针
println!("{}", r);    // 输出：5（自动解引用）
println!("{}", *r);   // 显式解引用：5

let mut y = 10;
let r_mut = &mut y;
*r_mut += 1;          // 通过解引用修改
println!("{}", *r_mut);  // 11
```

---

## 4. 模式匹配与错误处理

### match 表达式

```rust
let x = 5;

match x {
    0 => println!("zero"),
    1 => println!("one"),
    2..=4 => println!("two to four"),  // 范围
    _ => println!("other"),            // 默认分支
}

// match 作为表达式（有返回值）
let message = match x {
    0 => "zero",
    _ => "other",
};
println!("{}", message);

// 匹配结构体
struct Point { x: i32, y: i32 }
let p = Point { x: 1, y: 2 };

match p {
    Point { x: 0, y: 0 } => println!("origin"),
    Point { x, y } => println!("point: ({}, {})", x, y),
}
```

### if let（简化 match）

```rust
let x = Some(5);

if let Some(val) = x {
    println!("value: {}", val);  // 5
} else {
    println!("no value");
}

// 等价于
match x {
    Some(val) => println!("value: {}", val),
    None => println!("no value"),
}
```

### Result（错误处理）

```rust
// Result<T, E> 代表可能失败的操作
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("division by zero"))
    } else {
        Ok(a / b)
    }
}

// 处理方式 1：match
match divide(10, 2) {
    Ok(result) => println!("result: {}", result),
    Err(e) => println!("error: {}", e),
}

// 处理方式 2：unwrap（如果 Err 则 panic）
let result = divide(10, 2).unwrap();  // 10 / 2 = 5
let result = divide(10, 0).unwrap();  // panic!

// 处理方式 3：? 操作符（错误传播）
fn operation() -> Result<i32, String> {
    let a = divide(10, 2)?;  // 如果失败，立即返回 Err
    let b = divide(a, 2)?;
    Ok(b)
}

// 处理方式 4：unwrap_or（提供默认值）
let result = divide(10, 0).unwrap_or(-1);  // -1

// 处理方式 5：if let
if let Ok(result) = divide(10, 2) {
    println!("success: {}", result);
}
```

### Option（可能为空）

```rust
// Option<T> 代表值可能存在或不存在
let x: Option<i32> = Some(5);
let y: Option<i32> = None;

// 提取值
match x {
    Some(val) => println!("{}", val),
    None => println!("no value"),
}

// 使用 if let
if let Some(val) = x {
    println!("{}", val);
}

// 使用方法
let val = x.unwrap_or(0);  // 5 或 0
let val = x.unwrap_or_else(|| 0);  // 闭包版本
```

---

## 5. 特征（Trait）— 接口

### 定义与实现

```rust
// 定义特征
trait Animal {
    fn make_sound(&self);
    fn age(&self) -> u32;
}

// 为结构体实现特征
struct Dog {
    name: String,
    age_val: u32,
}

impl Animal for Dog {
    fn make_sound(&self) {
        println!("{} says: Woof!", self.name);
    }
    
    fn age(&self) -> u32 {
        self.age_val
    }
}

// 使用
let dog = Dog {
    name: "Rex".to_string(),
    age_val: 5,
};
dog.make_sound();
println!("age: {}", dog.age());
```

### 常见内置特征

```rust
// Debug — 用于调试输出
#[derive(Debug)]
struct Point { x: i32, y: i32 }

let p = Point { x: 1, y: 2 };
println!("{:?}", p);  // Point { x: 1, y: 2 }

// Display — 用于用户友好的输出（需手动实现）
use std::fmt;

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

println!("{}", p);  // (1, 2)

// Clone — 复制
#[derive(Clone)]
struct Data { val: i32 }

let d1 = Data { val: 10 };
let d2 = d1.clone();  // 深拷贝

// Serialize/Deserialize — 序列化（来自 serde 库）
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Message { text: String }
```

### 特征作为函数参数

```rust
fn print_sound<T: Animal>(animal: &T) {
    animal.make_sound();
}

let dog = Dog { /* ... */ };
print_sound(&dog);

// 或使用 dyn
fn print_sound_dyn(animal: &dyn Animal) {
    animal.make_sound();
}
```

---

## 6. 宏（Macro）

### 常见宏

```rust
// println! — 打印并换行
println!("Hello, {}!", "world");

// format! — 格式化字符串
let s = format!("Hello, {}!", "world");

// vec! — 创建向量
let v = vec![1, 2, 3];

// assert! — 断言（条件为 false 时 panic）
assert!(5 > 3);

// assert_eq! — 相等断言
assert_eq!(2 + 2, 4);

// panic! — 主动崩溃
panic!("Something went wrong!");

// dbg! — 调试输出
let x = 5;
dbg!(x);  // 输出：[src/main.rs:1] x = 5

// unwrap! — 提取 Result/Option，失败则 panic（实际上不是宏，是方法）
let Some(val) = Some(5) else { panic!() };
```

### 自定义宏（简单例子）

```rust
// 定义宏
macro_rules! add {
    ($a:expr, $b:expr) => {
        $a + $b
    };
}

// 使用
let result = add!(2, 3);  // 5
```

---

## 7. 控制流

### if / else

```rust
let x = 5;

if x > 0 {
    println!("positive");
} else if x < 0 {
    println!("negative");
} else {
    println!("zero");
}

// if 作为表达式（有返回值）
let msg = if x > 0 { "positive" } else { "not positive" };
```

### 循环

```rust
// while
let mut i = 0;
while i < 3 {
    println!("{}", i);
    i += 1;
}

// for（更常见）
for i in 0..3 {
    println!("{}", i);  // 0, 1, 2
}

// for with iter
let arr = vec![1, 2, 3];
for val in arr {
    println!("{}", val);
}

for (i, val) in arr.iter().enumerate() {
    println!("{}:{}", i, val);
}

// loop（无限循环）
let mut i = 0;
loop {
    if i >= 3 {
        break;
    }
    println!("{}", i);
    i += 1;
}

// continue（跳过本轮迭代）
for i in 0..5 {
    if i == 2 {
        continue;
    }
    println!("{}", i);  // 0, 1, 3, 4
}
```

---

## 8. 函数

### 定义与调用

```rust
// 基本函数
fn add(a: i32, b: i32) -> i32 {
    a + b  // 最后一行自动返回（无分号）
}

// 带分号的显式返回
fn add_explicit(a: i32, b: i32) -> i32 {
    return a + b;  // 分号表示语句
}

// 无返回值（返回 unit type ()）
fn print_num(n: i32) {
    println!("{}", n);
}

// 调用
let result = add(2, 3);  // 5
print_num(10);

// 提前返回
fn check_positive(x: i32) -> &'static str {
    if x < 0 {
        return "negative";
    }
    if x == 0 {
        return "zero";
    }
    "positive"
}
```

### 闭包（Lambda）

```rust
// 闭包定义与调用
let add = |a: i32, b: i32| a + b;
println!("{}", add(2, 3));  // 5

// 类型推导
let add = |a, b| a + b;

// 捕获环境变量
let x = 10;
let add_x = |a| a + x;
println!("{}", add_x(5));  // 15

// 可变闭包
let mut sum = 0;
let add_sum = |a: i32| {
    sum += a;
    sum
};
add_sum(1);
add_sum(2);
println!("{}", sum);  // 3

// 作为函数参数
fn apply<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32
{
    f(x)
}

let double = |x| x * 2;
println!("{}", apply(double, 5));  // 10
```

---

## 9. 导入与模块

### use 语句

```rust
// 导入具体项
use std::collections::HashMap;
use std::fmt::Debug;

// 导入多项
use std::io::{self, Read};  // self 代表 io 本身

// 通配符
use std::collections::*;  // 导入所有

// 别名
use std::collections::HashMap as Map;

// 相对导入
use crate::module_name::function;
use super::parent_module;

// 在函数中导入（作用域）
fn foo() {
    use std::collections::HashMap;
    // HashMap 只在这个函数中可用
}
```

### 模块

```rust
// 文件：src/lib.rs
mod utils {
    pub fn helper() {
        println!("help");
    }
}

mod tests {
    use super::*;  // 导入父模块所有公开项
    
    #[test]
    fn test_something() {
        // ...
    }
}

// 使用
utils::helper();

// 在子模块中访问父模块
mod child {
    pub fn call_parent() {
        super::utils::helper();  // super = 父模块
    }
}
```

---

## 10. 属性（Attribute）

### 常见属性

```rust
// 测试
#[test]
fn test_something() {
    assert!(true);
}

// 条件编译
#[cfg(test)]
mod tests {
    // 只在 cargo test 时编译
}

#[cfg(target_os = "windows")]
fn os_specific() {
    // 只在 Windows 上编译
}

// 派生特征
#[derive(Debug, Clone, Copy)]
struct Point { x: i32, y: i32 }

// 允许未使用的代码
#[allow(dead_code)]
fn unused() {}

// 已弃用
#[deprecated]
fn old_function() {}

// 内联优化
#[inline]
fn small_function() {}

// 处理宏警告
#![allow(unused_imports)]
```

---

## 11. 位操作（投票机中的关键）

### 基本操作

```rust
// 左移：a << n 等于 a * 2^n
1 << 0 = 1     // 0b0001
1 << 1 = 2     // 0b0010
1 << 2 = 4     // 0b0100
1 << 3 = 8     // 0b1000

// 右移：a >> n 等于 a / 2^n
8 >> 1 = 4
8 >> 2 = 2
8 >> 3 = 1

// 按位与：a & b（同为 1 则结果为 1）
0b1100 & 0b1010 = 0b1000

// 按位或：a | b（有 1 则结果为 1）
0b1100 | 0b1010 = 0b1110

// 按位异或：a ^ b（不同为 1）
0b1100 ^ 0b1010 = 0b0110

// 按位非：!a（反转所有位）
!0b1100 = 0b...0011（取决于类型宽度）
```

### 投票机的位操作例解

```rust
// 检查投票者 idx 是否已投票
let mut voter_bitfield: u32 = 0;

// 投票者 2 投票
let idx = 2;
let mask = 1 << idx;          // 1 << 2 = 0b0100 = 4
if 0 == voter_bitfield & mask {
    println!("投票者 {} 还未投票", idx);  // 0 & 4 = 0，成立
}

// 标记投票者 2 已投票
voter_bitfield |= mask;        // 0 | 4 = 4 = 0b0100

// 再次检查
if 0 == voter_bitfield & mask {
    println!("投票者 {} 还未投票", idx);  // 4 & 4 = 4 ≠ 0，不成立
} else {
    println!("投票者 {} 已投票", idx);     // 执行这里
}

// 检查投票者 1 是否投过票
let idx = 1;
let mask = 1 << idx;          // 1 << 1 = 0b0010 = 2
if 0 == voter_bitfield & mask {
    println!("投票者 {} 还未投票", idx);  // 4 & 2 = 0，成立
}
```

---

## 12. 引用 Crate 与依赖

### Cargo.toml 中的依赖

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
risc0-zkvm = "0.12"
```

### 在代码中导入

```rust
// 导入 crate 的根级项
use serde::{Serialize, Deserialize};
use risc0_zkvm::guest::env;

// 导入特征（使其方法可用）
use std::fmt::Display;

// 导入宏
#[macro_use]
extern crate serde;

// 重新导出（供其他 crate 使用）
pub use std::collections::HashMap;
```

---

## 13. 常见模式

### Builder 模式

```rust
struct ExecutorEnv {
    // ...
}

struct ExecutorEnvBuilder {
    input: Option<Vec<u8>>,
    output: Option<Vec<u8>>,
}

impl ExecutorEnvBuilder {
    fn new() -> Self {
        ExecutorEnvBuilder {
            input: None,
            output: None,
        }
    }
    
    fn write(mut self, data: &[u8]) -> Result<Self> {
        self.input = Some(data.to_vec());
        Ok(self)
    }
    
    fn stdout(mut self, output: &mut Vec<u8>) -> Result<Self> {
        self.output = Some(Vec::new());
        Ok(self)
    }
    
    fn build(self) -> Result<ExecutorEnv> {
        // 构建 ExecutorEnv
        Ok(ExecutorEnv {
            // ...
        })
    }
}

// 使用（方法链）
let env = ExecutorEnvBuilder::new()
    .write(&data)?
    .stdout(&mut output)?
    .build()?;
```

### RAII（资源获取即初始化）

```rust
struct Resource {
    id: i32,
}

impl Resource {
    fn new(id: i32) -> Self {
        println!("获取资源 {}", id);
        Resource { id }
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        println!("释放资源 {}", self.id);  // 作用域结束时自动调用
    }
}

{
    let _r = Resource::new(1);
    // ...
}  // 自动调用 drop，输出："释放资源 1"
```

---

## 速查索引

### 按概念查找

- **创建变量**：`let`, `let mut`, `const`
- **借用**：`&`, `&mut`, `*`（解引用）
- **转移所有权**：直接赋值或作为参数
- **复制**：`.clone()`
- **错误处理**：`Result<T, E>`, `Option<T>`, `?`, `unwrap()`
- **模式匹配**：`match`, `if let`
- **循环**：`for`, `while`, `loop`
- **函数指针与闭包**：`fn`, `|...|`
- **特征**：`trait`, `impl Trait for Type`
- **泛型**：`<T>`, `where T: Trait`
- **导入**：`use`
- **模块**：`mod`, `pub`
- **宏**：`macro_rules!`, `#[...]`

### 按错误类型查找

- **"value used after move"** → 检查所有权转移，使用 `&` 借用或 `.clone()`
- **"cannot borrow as mutable"** → 需要 `&mut self` 或 `let mut`
- **"expected &str, found String"** → 使用 `&string` 转换或 `.as_str()`
- **"no method named"** → 确保 trait 已导入或 struct 有该方法
- **"mismatched types"** → 检查类型转换，可能需要 `as`, `into()`, 等
- **"expected Result, found T"** → 需要 `Ok(T)` 或 `Err(E)`

---

## 投票机示例中的关键语法

### 以下是在本示例中实际出现的语法模式

```rust
// 1. 结构体定义与初始化
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VotingMachineState {
    pub polls_open: bool,
    pub voter_bitfield: u32,
    pub count: u32,
}

// 2. 实现方法
impl VotingMachineState {
    pub fn vote(&mut self, voter: u32, vote_yes: bool) -> bool {
        let voter_mask = 1 << voter;
        if self.polls_open && 0 == self.voter_bitfield & voter_mask {
            self.voter_bitfield |= voter_mask;
            if vote_yes { self.count += 1; }
            true
        } else {
            false
        }
    }
}

// 3. 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn protocol() {
        let mut polling_station = PollingStation::new(/* ... */);
        let result = polling_station.submit(&ballot).unwrap();
        assert_eq!(polling_station.state.count, 2);
    }
}

// 4. 日志宏
tracing::info!("Final vote count: {:?}", polling_station.state.count);

// 5. 错误处理
let env = ExecutorEnv::builder().write(&self.state)?.build()?;

// 6. 序列化
let state_bytes = to_vec(&state).unwrap();
let state_hash = *Impl::hash_words(&state_bytes);
```

---

## 总结表

| 概念 | 语法 | 例子 | 说明 |
|------|------|------|------|
| 不可变变量 | `let x = val;` | `let x = 10;` | 默认不可变 |
| 可变变量 | `let mut x = val;` | `let mut x = 10;` | 需要 `mut` 关键字 |
| 借用 | `&var` | `&x` | 创建引用，无所有权转移 |
| 可变借用 | `&mut var` | `&mut x` | 可以修改，同时只能一个 |
| 解引用 | `*ref` | `*r = 5` | 访问引用指向的值 |
| 结构体 | `struct Name { field: Type, ... }` | `struct Point { x: i32 }` | 定义数据结构 |
| 方法 | `fn method(&self)` | `fn is_open(&self)` | 不可变方法 |
| 可变方法 | `fn method(&mut self)` | `fn vote(&mut self)` | 可修改状态 |
| 模式匹配 | `match expr { pat => ... }` | `match x { 0 => ..., _ => ... }` | 条件分支 |
| Result 成功 | `Ok(val)` | `Ok(5)` | 表示操作成功 |
| Result 失败 | `Err(e)` | `Err("error")` | 表示操作失败 |
| 错误传播 | `expr?` | `func()?` | 失败时立即返回 |
| 特征 | `trait Name { ... }` | `trait Animal { ... }` | 定义接口 |
| 实现特征 | `impl Trait for Type` | `impl Animal for Dog` | 为类型实现特征 |
| 泛型 | `<T>` | `fn foo<T>(x: T)` | 通用类型参数 |
| 循环 | `for x in iter` | `for i in 0..5` | 迭代 |
| 位运算 | `a << n`, `a & b`, `a \| b` | `1 << 2`, `a & b` | 按位操作 |
| 属性 | `#[attr]` | `#[test]`, `#[derive(Debug)]` | 标注代码 |
| 宏 | `macro!()` | `println!()`, `assert_eq!()` | 编译时代码生成 |

---

祝学习愉快！如有任何疑问，回到 LEARNING_GUIDE.md 或 EXECUTION_TRACE.md 查看详细讲解。
