use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use data_validation::validate;

// 定义应用状态，包含 ID 映射
struct AppState {
    id_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() {
    // 初始化 ID 映射 (模拟配置文件)
    let mut id_map = HashMap::new();
    // 注意：这里的 ID 需要你运行 hello-world 后填入真实的 Hex 值
    id_map.insert("multiply".to_string(), "REPLACE_WITH_MULTIPLY_ID_HEX".to_string());
    id_map.insert("check_age".to_string(), "REPLACE_WITH_CHECK_AGE_ID_HEX".to_string());
    id_map.insert("check_hash".to_string(), "REPLACE_WITH_CHECK_HASH_ID_HEX".to_string());

    let state = Arc::new(RwLock::new(AppState { id_map }));

    let app = Router::new()
        .route("/validate", post(validate_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    println!("Validation server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct ValidateRequest {
    receipt_data: Vec<u8>,
    // 客户端可以传具体的 Hex ID，也可以传别名 (如 "check_age")
    image_id: String,
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    resolved_id: Option<String>, // 返回实际使用的 Hex ID
    error: Option<String>,
}

async fn validate_handler(
    axum::extract::State(state): axum::extract::State<Arc<RwLock<AppState>>>,
    Json(payload): Json<ValidateRequest>
) -> Json<ValidateResponse> {
    let state = state.read().await;

    // 1. 尝试解析 Image ID
    let image_id_hex = if let Some(hex) = state.id_map.get(&payload.image_id) {
        // 如果是别名，从映射中取
        hex.clone()
    } else {
        // 否则假设它本身就是 Hex
        payload.image_id.clone()
    };

    // 2. 调用验证逻辑
    match validate(&payload.receipt_data, &image_id_hex) {
        Ok(valid) => Json(ValidateResponse {
            valid,
            resolved_id: Some(image_id_hex),
            error: None
        }),
        Err(e) => Json(ValidateResponse {
            valid: false,
            resolved_id: Some(image_id_hex),
            error: Some(e)
        }),
    }
}
