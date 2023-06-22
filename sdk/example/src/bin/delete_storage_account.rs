use shadow_drive_sdk::ShadowDriveClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let storage_account_key =
        Pubkey::from_str("9VndNFwL7cVTshY2x5GAjKQusRCAsDU4zynYg76xTguo").unwrap();

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    let response = shdw_drive_client
        .delete_storage_account(&storage_account_key)
        .await
        .expect("failed to request storage account deletion");

    println!("Delete Storage Account complete {:?}", response);
}
