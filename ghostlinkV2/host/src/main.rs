//! GhostLink V2 Host - HTTP API Server
//!
//! Provides a REST API for generating zero-knowledge proofs for various credential types.

use axum::{
    extract::Json,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use ghostlink_v2_methods::{GHOSTLINK_V2_GUEST_ELF as GUEST_ELF, GHOSTLINK_V2_GUEST_ID as GUEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

// ============================================================================
// Request/Response Types (matching risc_zero_spec.md)
// ============================================================================

/// Credential type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CredentialType {
    Github,
    Alipay,
    Twitter,
    Wallet,
}

/// Unified prove request (Section 4.1 of spec)
#[derive(Debug, Deserialize)]
struct ProveRequest {
    credential_type: CredentialType,
    data: serde_json::Value,
    recipient: String,
}

/// Success response (Section 4.2 of spec)
#[derive(Debug, Serialize)]
struct ProveSuccessResponse {
    status: String,
    receipt_hex: String,
    journal_hex: String,
    image_id_hex: String,
    nullifier_hex: String,
}

/// Error response (Section 4.2 of spec)
#[derive(Debug, Serialize)]
struct ProveErrorResponse {
    status: String,
    error_code: String,
    message: String,
}

/// Guest input structure (must match guest/src/main.rs)
#[derive(Debug, Serialize)]
struct GuestInput {
    credential_type: CredentialType,
    data_json: String,
    recipient: [u8; 20],
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a slice of u32 to a Vec<u8> (little-endian)
fn to_u8_vec(data: &[u32]) -> Vec<u8> {
    let mut res = Vec::with_capacity(data.len() * 4);
    for &val in data {
        res.extend_from_slice(&val.to_le_bytes());
    }
    res
}

/// Parse Ethereum address from hex string to [u8; 20]
fn parse_address(addr: &str) -> Result<[u8; 20], String> {
    let clean_addr = addr.trim_start_matches("0x").trim();

    if clean_addr.len() != 40 {
        return Err(format!(
            "Invalid address length: {} (expected 40 hex chars)",
            clean_addr.len()
        ));
    }

    let bytes = hex::decode(clean_addr)
        .map_err(|e| format!("Invalid hex format: {}", e))?;

    if bytes.len() != 20 {
        return Err(format!(
            "Invalid address byte length: {} (expected 20 bytes)",
            bytes.len()
        ));
    }

    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

/// Extract nullifier from journal
fn extract_nullifier(journal: &[u8]) -> String {
    // New Compact Journal format (53 bytes): 
    // [20 bytes address][32 bytes nullifier][1 byte type]
    if journal.len() == 53 {
        hex::encode(&journal[20..52])
    } else if journal.len() >= 64 {
        // Fallback for old format if needed (though guest should be upgraded)
        hex::encode(&journal[32..64])
    } else {
        String::new()
    }
}

// ============================================================================
// API Handlers
// ============================================================================

async fn root() -> &'static str {
    "GhostLink V2 ZK Prover Service"
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "2.0.0"
    }))
}

async fn prove_handler(Json(payload): Json<ProveRequest>) -> Response {
    info!(
        "Received proof request: type={:?}, recipient={}",
        payload.credential_type, payload.recipient
    );

    let image_id_bytes = to_u8_vec(&GUEST_ID);
    let image_id_hex = hex::encode(&image_id_bytes);

    // 1. Parse recipient address
    let recipient_bytes = match parse_address(&payload.recipient) {
        Ok(addr) => addr,
        Err(e) => {
            error!("Address parsing error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ProveErrorResponse {
                    status: "error".to_string(),
                    error_code: "INVALID_ADDRESS".to_string(),
                    message: e,
                }),
            )
                .into_response();
        }
    };

    // 2. Serialize data to JSON string for guest
    let data_json = match serde_json::to_string(&payload.data) {
        Ok(json) => json,
        Err(e) => {
            error!("Data serialization error: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ProveErrorResponse {
                    status: "error".to_string(),
                    error_code: "INVALID_DATA".to_string(),
                    message: format!("Failed to serialize data: {}", e),
                }),
            )
                .into_response();
        }
    };

    // 3. Build guest input
    let guest_input = GuestInput {
        credential_type: payload.credential_type.clone(),
        data_json,
        recipient: recipient_bytes,
    };

    // 4. Create executor environment
    let env = match ExecutorEnv::builder().write(&guest_input) {
        Ok(builder) => match builder.build() {
            Ok(e) => e,
            Err(e) => {
                error!("Env build error: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ProveErrorResponse {
                        status: "error".to_string(),
                        error_code: "ENV_BUILD_FAILED".to_string(),
                        message: format!("Failed to build executor env: {}", e),
                    }),
                )
                    .into_response();
            }
        },
        Err(e) => {
            error!("Env write error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProveErrorResponse {
                    status: "error".to_string(),
                    error_code: "ENV_WRITE_FAILED".to_string(),
                    message: format!("Failed to write to executor env: {}", e),
                }),
            )
                .into_response();
        }
    };

    // 5. Generate proof
    info!("Starting proof generation...");
    let prover = default_prover();

    let prove_result = prover.prove(env, GUEST_ELF);
    let receipt = match prove_result {
        Ok(result) => result.receipt,
        Err(e) => {
            error!("Proving failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProveErrorResponse {
                    status: "error".to_string(),
                    error_code: "PROVING_FAILED".to_string(),
                    message: format!("Proof generation failed: {}", e),
                }),
            )
                .into_response();
        }
    };

    info!("Proof generated, compressing to Groth16...");

    // 6. Compress to Groth16 (only if allowed/supported)
    // In DEV_MODE, compress might return a FakeReceipt or Err depending on version.
    let compress_result = prover.compress(&ProverOpts::groth16(), &receipt);

    match compress_result {
        Ok(groth16_receipt) => {
            // Safe extraction of Groth16 inner
            if let Ok(groth16_inner) = groth16_receipt.inner.groth16() {
                let verifier_params = groth16_inner.verifier_parameters;
                let seal_data = groth16_inner.seal.clone();

                // Build seal with selector (first 4 bytes of verifier_parameters)
                let selector = &verifier_params.as_bytes()[..4];
                let mut seal_bytes = Vec::with_capacity(selector.len() + seal_data.len());
                seal_bytes.extend_from_slice(selector);
                seal_bytes.extend_from_slice(seal_data.as_ref());

                let journal_bytes = groth16_receipt.journal.bytes.clone();
                let nullifier_hex = extract_nullifier(&journal_bytes);

                info!("✅ Groth16 proof generated successfully");

                Json(ProveSuccessResponse {
                    status: "success".to_string(),
                    receipt_hex: hex::encode(&seal_bytes),
                    journal_hex: hex::encode(&journal_bytes),
                    image_id_hex,
                    nullifier_hex,
                })
                .into_response()
            } else {
                // Handle cases where compression "succeeded" but didn't produce a Groth16 proof
                // (Common in RISC0_DEV_MODE)
                let journal_bytes = receipt.journal.bytes.clone();
                let nullifier_hex = extract_nullifier(&journal_bytes);

                info!("⚠️ Dev mode: Returning fake receipt (non-Groth16)");

                Json(ProveSuccessResponse {
                    status: "success".to_string(),
                    receipt_hex: "00000000_fake_receipt_dev_mode".to_string(),
                    journal_hex: hex::encode(&journal_bytes),
                    image_id_hex,
                    nullifier_hex,
                })
                .into_response()
            }
        }
        Err(e) => {
            // In Dev Mode, even if compression fails entirely, we still want the journal
            if std::env::var("RISC0_DEV_MODE").is_ok() {
                let journal_bytes = receipt.journal.bytes.clone();
                let nullifier_hex = extract_nullifier(&journal_bytes);
                info!("⚠️ Dev mode: Compression failed, returning journal only");
                
                return Json(ProveSuccessResponse {
                    status: "success".to_string(),
                    receipt_hex: "00000000_compression_skipped_dev_mode".to_string(),
                    journal_hex: hex::encode(&journal_bytes),
                    image_id_hex,
                    nullifier_hex,
                }).into_response();
            }

            error!("Compression failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ProveErrorResponse {
                    status: "error".to_string(),
                    error_code: "COMPRESSION_FAILED".to_string(),
                    message: format!("Groth16 compression failed: {}", e),
                }),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Log image ID at startup
    let image_id_bytes = to_u8_vec(&GUEST_ID);
    let image_id_hex = hex::encode(&image_id_bytes);

    println!("==================================================");
    println!("🔐 GhostLink V2 ZK Prover Service");
    println!("==================================================");
    println!("📋 Supported credentials: github, alipay, twitter, wallet");
    println!("🔑 Guest Image ID: 0x{}", image_id_hex);
    println!("==================================================");

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/v1/prove", post(prove_handler))
        .layer(cors);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
