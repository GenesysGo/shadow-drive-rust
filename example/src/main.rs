use std::{io::SeekFrom, str::FromStr};

use byte_unit::Byte;
use shadow_drive_rust::{models::ShdwFile, Client};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use tokio::io::AsyncSeekExt;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let pubkey = keypair.pubkey();
    let (storage_account_key, _) =
        shadow_drive_rust::derived_addresses::storage_account(&pubkey, 0);

    //create shdw drive client
    let solana_rpc = RpcClient::new("https://ssc-dao.genesysgo.net");
    let shdw_drive_client = Client::new(keypair, solana_rpc)
        .await
        .expect("failed to create shdw drive client");

    let file = tokio::fs::File::open("example.png")
        .await
        .expect("failed to open file");

    let upload_reponse = shdw_drive_client
        .upload_file(
            &storage_account_key,
            ShdwFile {
                name: Some(String::from("example.png")),
                file,
            },
        )
        .await
        .expect("failed to upload file");

    println!("Upload complete {:?}", upload_reponse);
}

// let create_storage = shdw_drive_client
//     .create_storage_account(
//         "shdw-drive-rust-test",
//         Byte::from_str("69 MB").expect("failed to parse byte unit string"),
//     )
//     .await
//     .expect("Failed to create storage account");

// let storage_account = shdw_drive_client
//     .get_storage_account(&key)
//     .await
//     .expect("failed to get storage account");
// println!("{:#?}", storage_account.identifier);

// let accounts = shdw_drive_client
//     .get_storage_accounts(&storage_account.owner_1)
//     .await
//     .expect("failed to get storage accounts");

// for storage_account in accounts {
//     println!("{:#?}", storage_account.identifier);
// }

// let create_storage = shdw_drive_client
//     .create_storage_account(
//         "shdw-drive-rust-test",
//         Byte::from_str("69 MB").expect("failed to parse byte unit string"),
//     )
//     .await
//     .expect("Failed to create storage account");

// println!("{:#?}", create_storage);
