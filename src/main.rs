use bitcoin_hashes::sha256d;
use electrum_client::{bitcoin::Txid, Client, ElectrumApi};
use std::{error::Error, str::FromStr};

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
    for sibling_hash in merkle_path {
        let reversed = sibling_hash.iter().rev().cloned().collect::<Vec<u8>>();
        let mut combined = Vec::new();
        if cur_idx % 2 == 0 {
            // even -> compare from left
            combined.extend_from_slice(&current_hash);
            combined.extend_from_slice(&reversed);
        } else {
            // odd -> compare from right
            combined.extend_from_slice(&reversed);
            combined.extend_from_slice(&current_hash);
        }
        current_hash = sha256d::Hash::hash(&combined).to_byte_array();
        cur_idx /= 2;
    }

    // Final comparison to merkle root
    current_hash == *merkle_root
}

fn check_transaction_inclusion(height: usize, tx_id: &str) -> Result<bool, Box<dyn Error>> {
    let client = Client::new("tcp://electrum.blockstream.info:50001")?;
    let header = client.block_header(height)?;
    let txid = Txid::from_str(tx_id)?;
    let merkle_path = client.transaction_get_merkle(&txid, height)?;
    let txid: [u8; 32] = *txid.to_raw_hash().as_ref();
    Ok(validate_merkle_path(
        &txid,
        merkle_path.pos,
        merkle_path.merkle,
        header.merkle_root.as_ref(),
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let heights = [100000, 150000, 200000, 300000, 500000];
    let tx_ids = [
        "e9a66845e05d5abc0ad04ec80f774a7e585c6e8db975962d069a522137b80c1d",
        "25c6a1f8c0b5be2bee1e8dd3478b4ec8f54bbc3742eaf90bfb5afd46cf217ad9",
        "9ec5296ae83c24de706254122409d1164ebc58666962a4578372d4cc7ffebc30",
        "c33240a15d4e252ec0284e4079776843780a7ea8836bd91f8fb8217ca23eed9b",
        "f7bd6c0eb3c032eff48f41a06539cbfbf1be29514afb99b258696fc9dbd7efbc",
    ];
    for i in 0..heights.len() {
        let correct = check_transaction_inclusion(heights[i], tx_ids[i])?;
        println!(
            "Merkle path for {} at height {} is correct: {}",
            tx_ids[i], heights[i], correct
        );
    }
    Ok(())
}
