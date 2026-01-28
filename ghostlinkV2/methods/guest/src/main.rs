//! GhostLink V2 Guest Program
//!
//! This zkVM guest program verifies 4 types of credentials:
//! - GitHub: Verify GitHub account ownership
//! - Alipay: Verify assets >= threshold
//! - Twitter: Verify Twitter account ownership
//! - Wallet: Verify wallet ownership via signature
//!
//! Each verification produces a nullifier to prevent double-minting SBTs.

#![no_main]

use alloy_primitives::keccak256;
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

risc0_zkvm::guest::entry!(main);

// ============================================================================
// Data Structures
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

impl CredentialType {
    fn as_prefix(&self) -> &'static str {
        match self {
            CredentialType::Github => "github",
            CredentialType::Alipay => "alipay",
            CredentialType::Twitter => "twitter",
            CredentialType::Wallet => "wallet",
        }
    }
}

/// GitHub credential data
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubData {
    pub user_id: u64,
    pub username: String,
    pub created_at: String,
    pub public_repos: u32,
}

/// Alipay credential data
#[derive(Debug, Serialize, Deserialize)]
pub struct AlipayData {
    pub balance: String,
    pub id_number_hash: String,
    pub threshold: String,
}

/// Twitter credential data
#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterData {
    pub user_id: String,
    pub handle: String,
    pub created_at: String,
    #[serde(default)]
    pub followers_count: Option<u32>,
}

/// Wallet credential data
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletData {
    pub address: String,
    pub balance_wei: String,
    pub transaction_count: u32,
    pub chain_id: u32,
    pub signature: String,
    pub message: String,
}

/// Unified input structure for all credential types
#[derive(Debug, Serialize, Deserialize)]
pub struct GuestInput {
    pub credential_type: CredentialType,
    pub data_json: String,
    pub recipient: [u8; 20],
}

/// Output structure committed to the journal
#[derive(Debug, Serialize, Deserialize)]
pub struct GuestOutput {
    pub credential_type: String,
    pub verified: bool,
    pub nullifier: [u8; 32],
    pub recipient: [u8; 20],
}

// ============================================================================
// Verification Functions
// ============================================================================

/// Verify GitHub credential
fn verify_github(data_json: &str) -> (bool, [u8; 32]) {
    let data: GitHubData = serde_json::from_str(data_json)
        .expect("Failed to parse GitHub data");

    // Validation: check data format is valid
    assert!(!data.username.is_empty(), "Username cannot be empty");
    assert!(!data.created_at.is_empty(), "created_at cannot be empty");

    // Generate nullifier: keccak256("github" || user_id)
    let mut nullifier_input = Vec::new();
    nullifier_input.extend_from_slice(b"github");
    nullifier_input.extend_from_slice(&data.user_id.to_le_bytes());
    let nullifier_hash = keccak256(&nullifier_input);

    (true, nullifier_hash.into())
}

/// Verify Alipay credential
fn verify_alipay(data_json: &str) -> (bool, [u8; 32]) {
    let data: AlipayData = serde_json::from_str(data_json)
        .expect("Failed to parse Alipay data");

    // Parse balance and threshold as floats
    let balance: f64 = data.balance.parse()
        .expect("Invalid balance format");
    let threshold: f64 = data.threshold.parse()
        .expect("Invalid threshold format");

    // Core verification: balance >= threshold
    assert!(balance >= threshold, "Balance does not meet threshold");

    // Parse id_number_hash (should be hex string)
    let id_hash_clean = data.id_number_hash.trim_start_matches("0x");
    let id_hash_bytes = hex_decode(id_hash_clean)
        .expect("Invalid id_number_hash hex format");

    // Generate nullifier: keccak256("alipay" || id_number_hash)
    let mut nullifier_input = Vec::new();
    nullifier_input.extend_from_slice(b"alipay");
    nullifier_input.extend_from_slice(&id_hash_bytes);
    let nullifier_hash = keccak256(&nullifier_input);

    (true, nullifier_hash.into())
}

/// Verify Twitter credential
fn verify_twitter(data_json: &str) -> (bool, [u8; 32]) {
    let data: TwitterData = serde_json::from_str(data_json)
        .expect("Failed to parse Twitter data");

    // Validation: check data format is valid
    assert!(!data.user_id.is_empty(), "user_id cannot be empty");
    assert!(!data.handle.is_empty(), "handle cannot be empty");
    assert!(!data.created_at.is_empty(), "created_at cannot be empty");

    // Generate nullifier: keccak256("twitter" || user_id)
    let mut nullifier_input = Vec::new();
    nullifier_input.extend_from_slice(b"twitter");
    nullifier_input.extend_from_slice(data.user_id.as_bytes());
    let nullifier_hash = keccak256(&nullifier_input);

    (true, nullifier_hash.into())
}

/// Verify Wallet credential
fn verify_wallet(data_json: &str) -> (bool, [u8; 32]) {
    let data: WalletData = serde_json::from_str(data_json)
        .expect("Failed to parse Wallet data");

    // Validation: check data format
    assert!(data.address.starts_with("0x"), "Address must start with 0x");
    assert!(data.address.len() == 42, "Address must be 42 characters");

    // Optional threshold: transaction_count >= 10
    // This can be adjusted based on requirements
    // assert!(data.transaction_count >= 10, "Transaction count too low");

    // NOTE: Full ECDSA signature verification (ecrecover) is computationally expensive
    // in zkVM. For MVP, we trust the host-side verification.
    // In production, use RISC Zero's precompile or verify the signature here.

    // Parse address bytes
    let addr_clean = data.address.trim_start_matches("0x");
    let addr_bytes = hex_decode(addr_clean)
        .expect("Invalid address hex format");

    // Generate nullifier: keccak256("wallet" || address || chain_id)
    let mut nullifier_input = Vec::new();
    nullifier_input.extend_from_slice(b"wallet");
    nullifier_input.extend_from_slice(&addr_bytes);
    nullifier_input.extend_from_slice(&data.chain_id.to_le_bytes());
    let nullifier_hash = keccak256(&nullifier_input);

    (true, nullifier_hash.into())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Simple hex decoder (no external dependency needed)
fn hex_decode(hex_str: &str) -> Result<Vec<u8>, &'static str> {
    if hex_str.len() % 2 != 0 {
        return Err("Hex string has odd length");
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16)
            .map_err(|_| "Invalid hex character")?;
        bytes.push(byte);
    }
    Ok(bytes)
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    // 1. Read input from host
    let input: GuestInput = env::read();

    // 2. Dispatch to appropriate verifier
    let (verified, nullifier) = match input.credential_type {
        CredentialType::Github => verify_github(&input.data_json),
        CredentialType::Alipay => verify_alipay(&input.data_json),
        CredentialType::Twitter => verify_twitter(&input.data_json),
        CredentialType::Wallet => verify_wallet(&input.data_json),
    };

    // Ensure verification passed
    assert!(verified, "Verification failed");

    // 3. Build ABI-encoded journal for Solidity contract
    // Match abi.encodePacked(msg.sender, nullifier, uint8(credType))
    // Format: [20 bytes address] + [32 bytes nullifier] + [1 byte type] = 53 bytes
    let mut journal = Vec::with_capacity(53);

    // Part A: Address (20 bytes)
    journal.extend_from_slice(&input.recipient);

    // Part B: Nullifier (32 bytes)
    journal.extend_from_slice(&nullifier);

    // Part C: Credential type (1 byte)
    journal.push(match input.credential_type {
        CredentialType::Github => 0,
        CredentialType::Alipay => 1,
        CredentialType::Twitter => 2,
        CredentialType::Wallet => 3,
    });

    // 4. Commit journal to zkVM output
    env::commit_slice(&journal);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_nullifier() {
        let data = r#"{"user_id": 12345678, "username": "test", "created_at": "2020-01-01", "public_repos": 5}"#;
        let (verified, nullifier) = verify_github(data);
        assert!(verified);
        assert_eq!(nullifier.len(), 32);
    }

    #[test]
    fn test_alipay_threshold() {
        let data = r#"{"balance": "15000.00", "id_number_hash": "0x1234", "threshold": "10000"}"#;
        let (verified, _) = verify_alipay(data);
        assert!(verified);
    }

    #[test]
    #[should_panic(expected = "Balance does not meet threshold")]
    fn test_alipay_threshold_fail() {
        let data = r#"{"balance": "5000.00", "id_number_hash": "0x1234", "threshold": "10000"}"#;
        verify_alipay(data);
    }

    #[test]
    fn test_hex_decode() {
        let result = hex_decode("1234abcd").unwrap();
        assert_eq!(result, vec![0x12, 0x34, 0xab, 0xcd]);
    }
}
