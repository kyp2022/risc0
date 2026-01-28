use risc0_zkvm::{Receipt, Digest};

// 真正的验证函数
// receipt_bytes: 序列化后的 Receipt 数据 (bincode 格式)
// image_id_hex: Image ID 的 Hex 字符串
pub fn validate(receipt_bytes: &[u8], image_id_hex: &str) -> Result<bool, String> {
    // 1. 反序列化 Receipt
    let receipt: Receipt = bincode::deserialize(receipt_bytes)
        .map_err(|e| format!("Failed to deserialize receipt: {}", e))?;

    // 2. 解析 Image ID
    let image_id_bytes = hex::decode(image_id_hex)
        .map_err(|e| format!("Invalid hex string: {}", e))?;

    // 尝试将字节转换为 Digest
    let image_id = Digest::try_from(image_id_bytes.as_slice())
        .map_err(|_| "Image ID must be 32 bytes".to_string())?;

    // 3. 验证
    match receipt.verify(image_id) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Verification failed: {}", e)),
    }
}
