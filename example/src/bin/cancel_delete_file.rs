use shadow_drive_rust::ShadowDriveClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let storage_account_key =
        Pubkey::from_str("B7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT").unwrap();

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    let url = String::from(
        "https://shdw-drive.genesysgo.net/B7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT/hey.txt",
    );

    //cancel delete file
    let cancel_delete_file_response = shdw_drive_client
        .cancel_delete_file(&storage_account_key, url)
        .await
        .expect("failed to cancel file deletion");

    println!(
        "Unmark delete file complete {:?}",
        cancel_delete_file_response
    );
}
