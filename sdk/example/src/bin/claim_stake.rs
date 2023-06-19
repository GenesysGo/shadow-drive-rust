use shadow_drive_sdk::ShadowDriveClient;
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

/// This example doesn't quite work.
/// claim_stake is used to redeem SHDW after you reduce the storage amount of an account
/// In order to successfully claim_stake, the user needs to wait an epoch after reducing storage
/// Trying to claim_stake in the same epoch as a reduction will result in
/// "custom program error: 0x1775"
/// "Error Code: ClaimingStakeTooSoon"

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let storage_account_key =
        Pubkey::from_str("GHSNTDyMmay7xDjBNd9dqoHTGD3neioLk5VJg2q3fJqr").unwrap();

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    let url = String::from(
        "https://shdw-drive.genesysgo.net/B7Qk2omAvchkePhzHubCVQuVpZHcieqPQCwFxeeBZGuT/hey.txt",
    );

    // reduce storage

    // let reduce_storage_response = shdw_drive_client
    //     .reduce_storage(
    //         storage_account_key,
    //         Byte::from_str("100KB").expect("invalid byte string"),
    //     )
    //     .await
    //     .expect("error adding storage");

    // println!("txn id: {:?}", reduce_storage_response.txid);

    // WAIT AN EPOCH

    // claim stake
    // let claim_stake_response = shdw_drive_client
    //     .claim_stake(storage_account_key)
    //     .await
    //     .expect("failed to claim stake");

    // println!(
    //     "Claim stake complete {:?}",
    //     claim_stake_response
    // );
}
