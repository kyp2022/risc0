use risc0_zkvm::compute_image_id;
use std::fs;

fn main() {
    let elf_path = "target/riscv-guest/ghostlink-v2-methods/ghostlink-v2-guest/riscv32im-risc0-zkvm-elf/release/ghostlink-v2-guest";
    let elf = fs::read(elf_path).expect("failed to read elf");
    let image_id = compute_image_id(&elf).expect("failed to compute image id");
    println!("IMAGE_ID: {:?}", image_id);
    
    // Also print it as a hex string for easy use
    let hex_id: String = image_id.as_bytes().iter().map(|b| format!("{:02x}", b)).collect();
    println!("HEX_ID: 0x{}", hex_id);
}
