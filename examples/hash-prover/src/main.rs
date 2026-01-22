use axum::{
    routing::{get, post},
    Json, Router, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use risc0_zkvm::{default_prover, ExecutorEnv, Digest};
use hash_prover_methods::{HASH_GUEST_ELF, HASH_GUEST_ID};

#[derive(Deserialize)]
struct HashRequest {
    password: String,
}

#[derive(Serialize)]
struct HashResponse {
    receipt_data: Vec<u8>,
    image_id: String,
    hash_hex: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // 打印 Image ID
    println!("HASH_GUEST_ID: {}", Digest::from(HASH_GUEST_ID));

    let app = Router::new()
        .route("/", get(|| async { "Hash Prover Service Running!" }))
        .route("/prove", post(prove_hash));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Hash Prover listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn prove_hash(Json(payload): Json<HashRequest>) -> Result<Json<HashResponse>, (StatusCode, String)> {
    println!("收到哈希证明请求");

    let prove_info = tokio::task::spawn_blocking(move || {
        let env = ExecutorEnv::builder()
            .write(&payload.password).unwrap()
            .build()
            .map_err(|e| format!("Failed to build env: {}", e))?;

        let prover = default_prover();
        prover.prove(env, HASH_GUEST_ELF)
            .map_err(|e| format!("Proving failed: {}", e))
    }).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let receipt = prove_info.receipt;

    let hash_bytes = receipt.journal.bytes.clone();
    let hash_hex = hex::encode(hash_bytes);

    let receipt_bytes = bincode::serialize(&receipt)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let image_id_hex = Digest::from(HASH_GUEST_ID).to_string();

    Ok(Json(HashResponse {
        receipt_data: receipt_bytes,
        image_id: image_id_hex,
        hash_hex,
    }))
}
