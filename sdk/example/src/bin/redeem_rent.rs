use shadow_drive_sdk::ShadowDriveClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    let storage_account_key =
        Pubkey::from_str("D7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT").unwrap();

    let file_account_key =
        Pubkey::from_str("B41kFXqFkDhY7kHbMhEk17bP2w7QLUYU9X5tRhDLttnJ").unwrap();

    let redeem_rent_response = shdw_drive_client
        .redeem_rent(&storage_account_key, &file_account_key)
        .await
        .expect("failed to redeem_storage");
    println!("Redeemed {:?} \n", redeem_rent_response);
}
