use std::str::FromStr;

use bitcoin_hashes::sha256d;
use electrum_client::{bitcoin::Txid, Client, ElectrumApi};

// Function to validate the merkle path
fn validate_merkle_path(
    tx_hash: &[u8; 32],
    idx: usize,
    merkle_path: Vec<[u8; 32]>,
    merkle_root: &[u8; 32],
) -> bool {
    // Start with the transaction hash
    let mut current_hash = *tx_hash;

    let mut cur_idx = idx;
    let mut first_level = true;
    for sibling_hash in merkle_path {
        let reversed = sibling_hash.iter().rev().cloned().collect::<Vec<u8>>();
        let mut combined = Vec::new();
        if first_level && cur_idx % 2 == 0 {
            combined.extend_from_slice(&current_hash);
            combined.extend_from_slice(&reversed);
            println!(
                "{cur_idx} hashing {} with {}",
                hex::encode(current_hash),
                hex::encode(reversed)
            );
        } else {
            combined.extend_from_slice(&reversed);
            combined.extend_from_slice(&current_hash);
            println!(
                "{cur_idx} hashing {} with {}",
                hex::encode(reversed),
                hex::encode(current_hash),
            );
        }
        current_hash = sha256d::Hash::hash(&combined).to_byte_array();
        cur_idx /= 2;
        first_level = false;
    }

    // Final comparison to merkle root
    current_hash == *merkle_root
}

const BLOCK_HEIGHT: usize = 300000;
const TX_ID: &str = "c33240a15d4e252ec0284e4079776843780a7ea8836bd91f8fb8217ca23eed9b";
// const BLOCK_HEIGHT: usize = 100000;
// const TX_ID: &str = "e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new("tcp://electrum.blockstream.info:50001")?;
    let response = client.server_features()?;
    println!("{:?}", response);
    let header = client.block_header(BLOCK_HEIGHT)?;
    println!("{:?}", header);
    let txid = Txid::from_str(TX_ID)?;
    let merkle_path = client.transaction_get_merkle(&txid, BLOCK_HEIGHT)?;
    println!("{:?}", merkle_path);
    let txid: [u8; 32] = *txid.to_raw_hash().as_ref();
    let correct = validate_merkle_path(
        &txid,
        merkle_path.pos,
        merkle_path.merkle,
        header.merkle_root.as_ref(),
    );
    println!("Merkle path is correct: {}", correct);
    Ok(())
}
