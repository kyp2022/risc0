#![no_main]
#![no_std]

use risc0_zkvm::guest::env;
use serde::Deserialize;

risc0_zkvm::guest::entry!(main);

// 定义输入结构体
// 必须派生 Deserialize 才能从 env::read() 中读取
#[derive(Deserialize)]
struct UserData {
    current_year: u32,
    birth_year: u32,
    country_code: u32, // 新增字段：国家代码
}

fn main() {
    // 1. 读取结构体数据
    // env::read() 会自动反序列化 Host 传入的数据
    let input: UserData = env::read();

    // 2. 基础逻辑检查
    if input.birth_year > input.current_year {
        panic!("Invalid birth year: future date");
    }

    let age = input.current_year - input.birth_year;

    // --- 任务 B: 国家代码检查 ---
    // 只允许 country_code 为 86 (中国)
    if input.country_code != 86 {
        panic!("Invalid country code: only 86 is allowed");
    }

    // --- 任务 A: 年龄上限检查 ---
    // 年龄不能超过 100 岁
    if age > 100 {
        panic!("Invalid age: too old (> 100)");
    }

    // 3. 核心业务逻辑: 必须满 18 岁
    if age < 18 {
        panic!("Underage");
    }

    // 4. 提交公开结果 (只提交当前年份)
    env::commit(&input.current_year);
    env::commit(&age);
}
