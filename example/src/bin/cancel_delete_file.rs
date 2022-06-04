use byte_unit::Byte;
use shadow_drive_rust::{models::ShdwFile, Client};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let storage_account_key =
        Pubkey::from_str("B7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT").unwrap();

    //create shdw drive client
    let solana_rpc = RpcClient::new("https://ssc-dao.genesysgo.net");
    let shdw_drive_client = Client::new(keypair, solana_rpc);

    let url = String::from(
        "https://shdw-drive.genesysgo.net/B7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT/hey.txt",
    );

    //cancel delete file
    let cancel_delete_file_response = shdw_drive_client
        .cancel_delete_file(storage_account_key, url)
        .await
        .expect("failed to request storage account deletion");

    println!("Unmark delete file complete {:?}", cancel_delete_file_response);
}
