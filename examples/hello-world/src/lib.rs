

// 将README.md文件的内容作为文档字符串包含进来
#![doc = include_str!("../README.md")]

// 导入MULTIPLY_ELF，这是编译后的zkVM guest程序（ELF格式）
use hello_world_methods::MULTIPLY_ELF;
// 从risc0_zkvm导入相关类型：默认prover、执行环境和收据类型
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt, Result};
// 导入Digest类型，用于将u32数组转换为Digest
use risc0_zkvm::Digest;

// 这是RISC Zero zkVM的Hello World演示。
// 通过运行此演示，Alice可以生成一个收据，证明她知道
// 一些数字a和b，使得a*b == 391。
// 因子a和b是保密的。

// 在zkVM内部计算乘积a*b
pub fn multiply(a: u64, b: u64) -> (Receipt, u64) {
    // 创建执行环境构建器，用于配置zkVM执行环境
    let env = ExecutorEnv::builder()
        // 将参数a和b写入到zkVM环境中，供guest程序读取
        .write(&a)        // 将第一个参数a写入环境
        .unwrap()         // 如果写入失败则panic
        .write(&b)        // 将第二个参数b写入环境
        .unwrap()         // 如果写入失败则panic
        .build()          // 构建执行环境
        .unwrap();        // 如果构建失败则panic

    // 获取默认的prover（证明器）实例
    let prover = default_prover();

    // 使用prover证明指定的ELF二进制文件执行的正确性，生成收据
    // MULTIPLY_ELF是预编译的zkVM guest程序，执行乘法运算
    let receipt = prover.prove(env, MULTIPLY_ELF).unwrap().receipt;

    // 从收据的journal中解码输出结果（即乘积c，其中c = a * b）
    // journal包含zkVM程序的公共输出
    let c: u64 = receipt.journal.decode().expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    // 打印结果，表明可以证明知道这些因子
    println!("I know the factors of {c}, and I can prove it!");

    // 打印收据
    println!("Receipt11: {:?}", receipt);



    // 返回证明收据和计算结果
    (receipt, c)
}

/**
 * 验证收据的正确性
 *
 * 此函数接收一个Receipt和一个Image ID（以u32数组形式），
 * 然后验证收据是否有效。
 *
 * # 参数
 *
 * * `receipt` - 要验证的收据
 * * `image_id` - Guest程序的Image ID，格式为[u32; 8]
 *
 * # 返回
 *
 * 如果验证成功，返回Ok(())；如果验证失败，返回错误信息。
 *
 * # 示例
 *
 * ```rust,no_run
 * use hello_world::{verify_receipt, multiply};
 * use hello_world_methods::MULTIPLY_ID;
 *
 * // 生成收据
 * let (receipt, _) = multiply(17, 23);
 *
 * // 验证收据
 * verify_receipt(&receipt, MULTIPLY_ID)?;
 * # Ok::<(), Box<dyn std::error::Error>>(())
 * ```
 */
pub fn verify_receipt(receipt: &Receipt, image_id: [u32; 8]) -> Result<()> {
    // 将u32数组转换为Digest类型
    let digest: Digest = Digest::from(image_id);

    // 验证收据并返回结果
    receipt.verify(digest)?;
    Ok(())
}

/**
 * 从文件加载收据
 *
 * 从指定路径的文件中读取序列化的收据（支持 JSON 或 bincode 格式）
 *
 * # 参数
 *
 * * `file_path` - 收据文件的路径
 *
 * # 返回
 *
 * 如果成功，返回 Receipt；如果失败，返回错误信息
 *
 * # 示例
 *
 * ```rust,no_run
 * use hello_world::load_receipt_from_file;
 *
 * let receipt = load_receipt_from_file("receipt.json")?;
 * # Ok::<(), Box<dyn std::error::Error>>(())
 * ```
 */
pub fn load_receipt_from_file(file_path: &str) -> Result<Receipt> {
    use std::fs;

    let content = fs::read(file_path)?;
    load_receipt_from_bytes(&content)
}

/**
 * 从字节数组加载收据
 *
 * 从字节数组中反序列化收据（支持 JSON 或 bincode 格式）
 *
 * # 参数
 *
 * * `data` - 序列化的收据数据（字节数组）
 *
 * # 返回
 *
 * 如果成功，返回 Receipt；如果失败，返回错误信息
 *
 * # 示例
 *
 * ```rust,no_run
 * use hello_world::load_receipt_from_bytes;
 *
 * // 假设我们有一个收据的字节数组
 * let receipt_bytes: Vec<u8> = vec![0, 1, 2, 3];
 * let receipt = load_receipt_from_bytes(&receipt_bytes)?;
 * # Ok::<(), Box<dyn std::error::Error>>(())
 * ```
 */
pub fn load_receipt_from_bytes(data: &[u8]) -> Result<Receipt> {
    // 尝试从 JSON 反序列化
    if let Ok(receipt) = serde_json::from_slice::<Receipt>(data) {
        return Ok(receipt);
    }

    // 如果 JSON 失败，尝试 bincode
    match bincode::deserialize::<Receipt>(data) {
        Ok(receipt) => Ok(receipt),
        Err(e) => Err(anyhow::anyhow!("Failed to deserialize receipt from bytes: {}", e)),
    }
}

// 测试模块，只在测试编译时包含
#[cfg(test)]
mod tests {
    // 导入父模块的所有公开项
    use super::*;

    // 测试函数，验证hello world示例是否正常工作
    #[test]
    fn test_hello_world() {
        // 定义测试用的两个因子常量
        const TEST_FACTOR_ONE: u64 = 17;
        const TEST_FACTOR_TWO: u64 = 23;

        // 调用multiply函数，忽略收据，只获取结果
        let (_, result) = multiply(17, 23);

        // 断言结果等于两个因子的乘积
        assert_eq!(
            result,
            TEST_FACTOR_ONE * TEST_FACTOR_TWO,  // 期望值：17 * 23 = 391
            "We expect the zkVM output to be the product of the inputs"  // 错误消息
        )
    }

    // 测试函数，验证verify_receipt函数是否正常工作
    #[test]
    fn test_verify_receipt() {
        use hello_world_methods::MULTIPLY_ID;

        // 生成收据
        let (receipt, _) = multiply(17, 23);

        // 验证收据
        verify_receipt(&receipt, MULTIPLY_ID).expect("Receipt verification should succeed");
    }
}
