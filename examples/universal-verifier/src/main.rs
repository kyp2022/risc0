use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use risc0_zkvm::{Receipt, Digest};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/validate", post(validate_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Universal Verifier listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct ValidateRequest {
    receipt_data: Vec<u8>,
    image_id: String, // 客户端必须传 Hex 格式的 Image ID
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    journal_hex: Option<String>, // 返回 Journal 的 Hex 表示，方便调试
    error: Option<String>,
}

async fn validate_handler(Json(payload): Json<ValidateRequest>) -> Json<ValidateResponse> {
    // 1. 反序列化 Receipt
    let receipt: Receipt = match bincode::deserialize(&payload.receipt_data) {
        Ok(r) => r,
        Err(e) => return Json(ValidateResponse {
            valid: false,
            journal_hex: None,
            error: Some(format!("Failed to deserialize receipt: {}", e))
        }),
    };

    // 2. 解析 Image ID
    let image_id_bytes = match hex::decode(&payload.image_id) {
        Ok(b) => b,
        Err(e) => return Json(ValidateResponse {
            valid: false,
            journal_hex: None,
            error: Some(format!("Invalid hex image_id: {}", e))
        }),
    };

    let image_id = match Digest::try_from(image_id_bytes.as_slice()) {
        Ok(d) => d,
        Err(_) => return Json(ValidateResponse {
            valid: false,
            journal_hex: None,
            error: Some("Image ID must be 32 bytes".to_string())
        }),
    };

    // 3. 验证
    match receipt.verify(image_id) {
        Ok(_) => {
            let journal_hex = hex::encode(&receipt.journal.bytes);
            Json(ValidateResponse {
                valid: true,
                journal_hex: Some(journal_hex),
                error: None
            })
        },
        Err(e) => Json(ValidateResponse {
            valid: false,
            journal_hex: None,
            error: Some(format!("Verification failed: {}", e))
        }),
    }
}
