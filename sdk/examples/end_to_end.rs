use std::str::FromStr;

use shadow_drive_sdk::{models::ShadowFile, Byte, Keypair, Pubkey, ShadowDriveClient};
use solana_sdk::signature::read_keypair_file;

const SOLANA_MAINNET_RPC: &'static str = "https://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() {
    // Get keypair
    let keypair_file: String = std::env::args()
        .skip(1)
        .next()
        .expect("no keypair file provided");
    let keypair: Keypair = read_keypair_file(keypair_file).expect("failed to read keypair file");
    println!("loaded keypair");

    // Initialize client
    let client = ShadowDriveClient::new(keypair, SOLANA_MAINNET_RPC);
    println!("initialized client");

    // Create account
    let response = client
        .create_storage_account(
            "test",
            Byte::from_bytes(2_u128.pow(20)),
            shadow_drive_sdk::StorageAccountVersion::V2,
        )
        .await
        .expect("failed to create storage account");
    let account = Pubkey::from_str(&response.shdw_bucket.unwrap()).unwrap();
    println!("created storage account");

    // Upload files
    let files: Vec<ShadowFile> = vec![
        ShadowFile::file("alpha.txt".to_string(), "./examples/files/alpha.txt"),
        ShadowFile::file(
            "not_alpha.txt".to_string(),
            "./examples/files/not_alpha.txt",
        ),
    ];
    let response = client
        .store_files(&account, files.clone())
        .await
        .expect("failed to upload files");
    println!("uploaded files");
    for url in &response.finalized_locations {
        println!("    {url}")
    }
    // Try editing
    for file in files {
        let response = client
            .edit_file(&account, file)
            .await
            .expect("failed to upload files");
        assert!(!response.finalized_location.is_empty(), "failed edit");
        println!("edited file: {}", response.finalized_location);
    }

    // Delete files
    for url in response.finalized_locations {
        client
            .delete_file(&account, url)
            .await
            .expect("failed to delete files");
    }
    println!("deleted files");

    // Delete account
    client
        .delete_storage_account(&account)
        .await
        .expect("failed to delete storage account");
    println!("deleted account");
}
