use axum::{
      extract::Json,
      http::Method,
      response::{IntoResponse, Response},
      routing::{get, post},
      Router,
};
use methods::{GHOSTLINK_GUEST_ELF, GHOSTLINK_GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct ProveRequest {
      input_json: String,
      recipient: String,
}

#[derive(Serialize)]
struct ProveResponse {
      receipt_hex: String,
      journal_hex: String,
      image_id_hex: String,
      nullifier_hex: String, // New field
      status: String,
}

#[derive(Serialize)]
struct GuestInput {
      user_json: String,
      recipient: [u8; 20],
}

fn to_u8_vec(data: &[u32]) -> Vec<u8> {
      let mut res = Vec::with_capacity(data.len() * 4);
      for &val in data {
            res.extend_from_slice(&val.to_le_bytes());
      }
      res
}

fn parse_address(addr: &str) -> Result<[u8; 20], String> {
      let clean_addr = addr.trim_start_matches("0x").trim();

      // 验证地址长度（应该是40个十六进制字符）
      if clean_addr.len() != 40 {
            return Err(format!("Invalid address length: {} (expected 40 hex chars). Address: {}", clean_addr.len(), addr));
      }

      let bytes = match hex::decode(clean_addr) {
            Ok(b) => b,
            Err(e) => return Err(format!("Invalid hex format: {}. Address: {}", e, addr)),
      };

      if bytes.len() != 20 {
            return Err(format!("Invalid address byte length: {} (expected 20 bytes). Address: {}", bytes.len(), addr));
      }

      let mut arr = [0u8; 20];
      arr.copy_from_slice(&bytes);
      Ok(arr)
}

#[tokio::main]
async fn main() {
      tracing_subscriber::fmt::init();

      let image_id_bytes = to_u8_vec(&GHOSTLINK_GUEST_ID);
      let image_id_hex = hex::encode(&image_id_bytes);

      println!("--------------------------------------------------");
      println!("🔑 GUEST IMAGE ID: 0x{}", image_id_hex);
      println!("--------------------------------------------------");

      let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any);

      let app = Router::new()
            .route("/", get(root))
            .route("/prove", post(prove_handler))
            .layer(cors);

      let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
      println!("🚀 Prover Service (Groth16) running at http://{}", addr);
      let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
      axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
      "GhostLink Groth16 Prover Service"
}

async fn prove_handler(Json(payload): Json<ProveRequest>) -> Response {
      println!("Received proof request for recipient: {}", payload.recipient);
      println!("Recipient address length: {} chars", payload.recipient.len());

      let image_id_bytes = to_u8_vec(&GHOSTLINK_GUEST_ID);
      let image_id_hex = hex::encode(&image_id_bytes);

      // 验证并解析地址
      let recipient_bytes = match parse_address(&payload.recipient) {
            Ok(addr) => addr,
            Err(e) => {
                  println!("❌ Address parsing error: {}", e);
                  return Json(ProveResponse {
                        receipt_hex: "".to_string(),
                        journal_hex: "".to_string(),
                        image_id_hex,
                        nullifier_hex: "".to_string(),
                        status: format!("Invalid recipient address: {}", e),
                  }).into_response();
            }
      };

      println!("✅ Parsed recipient address: {} bytes", recipient_bytes.len());
      let guest_input = GuestInput {
            user_json: payload.input_json,
            recipient: recipient_bytes,
      };

      let env = ExecutorEnv::builder()
            .write(&guest_input)
            .unwrap()
            .build()
            .unwrap();

      let prover = default_prover();

      let prove_result = prover.prove(env, GHOSTLINK_GUEST_ELF);
      if let Err(e) = prove_result {
            return Json(ProveResponse {
                  receipt_hex: "".to_string(),
                  journal_hex: "".to_string(),
                  image_id_hex: image_id_hex,
                  nullifier_hex: "".to_string(),
                  status: format!("Proving failed: {}", e),
            }).into_response();
      }
      let receipt = prove_result.unwrap().receipt;

      println!("Raw Journal Length: {} bytes", receipt.journal.bytes.len());
      println!("Journal (hex): {}", hex::encode(&receipt.journal.bytes));

      // Extract Nullifier (Last 32 bytes of journal)
      // Journal format: [12 bytes padding][20 bytes address][32 bytes nullifier] = 64 bytes total
      let journal_len = receipt.journal.bytes.len();
      let nullifier_hex = if journal_len >= 32 {
            // Nullifier is the last 32 bytes
            hex::encode(&receipt.journal.bytes[journal_len - 32..])
      } else {
            eprintln!("Warning: Journal too short, expected 64 bytes, got {} bytes", journal_len);
            "".to_string()
      };

      println!("Extracted Nullifier: {}", nullifier_hex);

      println!("Compressing proof to Groth16...");

      let compress_result = prover.compress(&ProverOpts::groth16(), &receipt);

      match compress_result {
            Ok(groth16_receipt) => {
                  // 使用 encode_seal 函数正确编码 seal（包含选择器）
                  // 这会将 verifier_parameters 的前4个字节作为选择器添加到 seal 前面
                  let groth16_inner = groth16_receipt.inner.groth16().unwrap();
                  let verifier_params = groth16_inner.verifier_parameters;
                  let seal_data = groth16_inner.seal.clone();

                  // 手动构建包含选择器的 seal
                  // 选择器是 verifier_parameters 的前4个字节
                  let selector = &verifier_params.as_bytes()[..4];
                  let mut seal_bytes = Vec::with_capacity(selector.len() + seal_data.len());
                  seal_bytes.extend_from_slice(selector);
                  seal_bytes.extend_from_slice(seal_data.as_ref());

                  let journal_bytes = groth16_receipt.journal.bytes.clone();

                  let receipt_hex = hex::encode(&seal_bytes);
                  let journal_hex = hex::encode(&journal_bytes);

                  println!("✅ Groth16 Proof Generated!");
                  println!("Selector (first 4 bytes): {}", hex::encode(selector));
                  println!("Seal length: {} bytes ({} selector + {} proof)", seal_bytes.len(), selector.len(), seal_data.len());
                  println!("Journal length: {} bytes", journal_bytes.len());

                  Json(ProveResponse {
                        receipt_hex,
                        journal_hex,
                        image_id_hex,
                        nullifier_hex,
                        status: "success".to_string(),
                  }).into_response()
            }
            Err(e) => {
                  println!("❌ Compression failed: {}", e);
                  Json(ProveResponse {
                        receipt_hex: "".to_string(),
                        journal_hex: "".to_string(),
                        image_id_hex,
                        nullifier_hex: "".to_string(),
                        status: format!("Compression failed: {}", e),
                  }).into_response()
            }
      }
}

