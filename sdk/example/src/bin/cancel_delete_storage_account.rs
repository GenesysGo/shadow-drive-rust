use shadow_drive_sdk::ShadowDriveClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let storage_account_key =
        Pubkey::from_str("GHSNTDyMmay7xDjBNd9dqoHTGD3neioLk5VJg2q3fJqr").unwrap();

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    let response = shdw_drive_client
        .cancel_delete_storage_account(&storage_account_key)
        .await
        .expect("failed to cancel storage account deletion");

    println!("Unmark delete storage account complete {:?}", response);
}
