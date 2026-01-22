use axum::{
    routing::{get, post},
    Json, Router, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt, Digest};
use hello_world_methods::{MULTIPLY_ELF, MULTIPLY_ID};
use hello_world_methods::{CHECK_AGE_ELF, CHECK_AGE_ID};
// 引入新的常量 (注意：如果编译报错找不到，请先注释掉这行和下面的相关路由，运行一次 cargo build)
use hello_world_methods::{CHECK_HASH_ELF, CHECK_HASH_ID};

// --- 基础结构体 ---
#[derive(Deserialize)]
struct ProveRequest { a: u64, b: u64 }

#[derive(Serialize)]
struct ProveResponse { receipt_data: Vec<u8>, image_id: String, product: u64 }

#[derive(Deserialize)]
struct VerifyRequest { receipt_data: Vec<u8> }

#[derive(Serialize)]
struct VerifyResponse { valid: bool, journal_output: Option<String>, error: Option<String> }

// --- 年龄验证结构体 ---
#[derive(Deserialize, Serialize)]
#[allow(dead_code)]
struct AgeCheckRequest { current_year: u32, birth_year: u32, country_code: u32 }

#[derive(Serialize)]
#[allow(dead_code)]
struct AgeCheckResponse { receipt_data: Vec<u8>, verified_year: u32 }

// --- 哈希验证结构体 (任务 C) ---
#[derive(Deserialize)]
struct HashRequest {
    password: String,
}

#[derive(Serialize)]
struct HashResponse {
    receipt_data: Vec<u8>,
    hash_hex: String, // 返回计算出的哈希值
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("MULTIPLY_ID: {}", Digest::from(MULTIPLY_ID));
    println!("CHECK_AGE_ID: {}", Digest::from(CHECK_AGE_ID));
    println!("CHECK_HASH_ID: {}", Digest::from(CHECK_HASH_ID));

    let app = Router::new()
        .route("/", get(|| async { "RISC Zero Server Running!" }))
        .route("/prove", post(generate_proof))
        .route("/verify", post(verify_proof))
        .route("/prove_age", post(prove_age))
        .route("/verify_age", post(verify_age))
        // 新增路由
        .route("/prove_hash", post(prove_hash))
        .route("/verify_hash", post(verify_hash));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

// ... (generate_proof, verify_proof, prove_age, verify_age 保持不变，为了节省篇幅省略，实际文件中应保留) ...
// 为了保证文件完整性，我必须把之前的代码也写回去，否则会覆盖丢失。
// 下面是完整的代码内容：

async fn generate_proof(Json(payload): Json<ProveRequest>) -> Result<Json<ProveResponse>, (StatusCode, String)> {
    let prove_info = tokio::task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&payload.a).unwrap()
            .write(&payload.b).unwrap()
            .build().unwrap();
        default_prover().prove(env, MULTIPLY_ELF)
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let receipt = prove_info.receipt;
    let product: u64 = receipt.journal.decode().unwrap();
    let receipt_bytes = bincode::serialize(&receipt).unwrap();
    let image_id_hex = Digest::from(MULTIPLY_ID).to_string();

    Ok(Json(ProveResponse { receipt_data: receipt_bytes, image_id: image_id_hex, product }))
}

async fn verify_proof(Json(payload): Json<VerifyRequest>) -> Json<VerifyResponse> {
    let receipt: Receipt = bincode::deserialize(&payload.receipt_data).unwrap();
    match receipt.verify(MULTIPLY_ID) {
        Ok(_) => Json(VerifyResponse { valid: true, journal_output: Some(receipt.journal.decode::<u64>().unwrap().to_string()), error: None }),
        Err(e) => Json(VerifyResponse { valid: false, journal_output: None, error: Some(e.to_string()) }),
    }
}

async fn prove_age(Json(payload): Json<AgeCheckRequest>) -> Result<Json<AgeCheckResponse>, (StatusCode, String)> {
    let prove_info = tokio::task::spawn_blocking(move || {
        let env = ExecutorEnv::builder().write(&payload).unwrap().build().unwrap();
        default_prover().prove(env, CHECK_AGE_ELF)
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let receipt = prove_info.receipt;
    let verified_year: u32 = receipt.journal.decode().unwrap();
    let receipt_bytes = bincode::serialize(&receipt).unwrap();

    Ok(Json(AgeCheckResponse { receipt_data: receipt_bytes, verified_year }))
}

async fn verify_age(Json(payload): Json<VerifyRequest>) -> Json<VerifyResponse> {
    let receipt: Receipt = bincode::deserialize(&payload.receipt_data).unwrap();
    match receipt.verify(CHECK_AGE_ID) {
        Ok(_) => Json(VerifyResponse { valid: true, journal_output: Some(receipt.journal.decode::<u32>().unwrap().to_string()), error: None }),
        Err(e) => Json(VerifyResponse { valid: false, journal_output: None, error: Some(e.to_string()) }),
    }
}

// --- 任务 C: 哈希证明实现 ---

async fn prove_hash(Json(payload): Json<HashRequest>) -> Result<Json<HashResponse>, (StatusCode, String)> {
    println!("收到哈希证明请求，密码长度: {}", payload.password.len());

    let prove_info = tokio::task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&payload.password).unwrap()
            .build()
            .map_err(|e| format!("Failed to build env: {}", e))?;

        let prover = default_prover();
        prover.prove(env, CHECK_HASH_ELF)
            .map_err(|e| format!("Proving failed: {}", e))
    }).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let receipt = prove_info.receipt;

    // Journal 中包含的是 [u8; 32] 的哈希值
    // 我们直接获取 bytes 并转为 hex
    let hash_bytes = receipt.journal.bytes.clone();
    let hash_hex = hex::encode(hash_bytes);

    let receipt_bytes = bincode::serialize(&receipt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(HashResponse {
        receipt_data: receipt_bytes,
        hash_hex,
    }))
}

async fn verify_hash(Json(payload): Json<VerifyRequest>) -> Json<VerifyResponse> {
    let receipt: Receipt = match bincode::deserialize(&payload.receipt_data) {
        Ok(r) => r,
        Err(e) => return Json(VerifyResponse { valid: false, journal_output: None, error: Some(e.to_string()) }),
    };

    match receipt.verify(CHECK_HASH_ID) {
        Ok(_) => {
            let hash_hex = hex::encode(&receipt.journal.bytes);
            Json(VerifyResponse { valid: true, journal_output: Some(hash_hex), error: None })
        },
        Err(e) => Json(VerifyResponse { valid: false, journal_output: None, error: Some(e.to_string()) }),
    }
}
