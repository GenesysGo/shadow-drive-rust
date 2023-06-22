use byte_unit::Byte;
use shadow_drive_sdk::{
    models::storage_acct::StorageAcct, ShadowDriveClient, StorageAccountVersion,
};
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "/Users/dboures/.config/solana/id.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    // // 1.
    // create_storage_accounts(shdw_drive_client).await;

    // // 2.
    // let v1_pubkey = Pubkey::from_str("J4RJYandDDKxyF6V1HAdShDSbMXk78izZ2yEksqyvGmo").unwrap();
    let v2_pubkey = Pubkey::from_str("9dXUV4BEKWohSRDn4cy5G7JkhUDWoSUGGwJngrSg453r").unwrap();

    // make_storage_immutable(&shdw_drive_client, &v1_pubkey).await;
    // make_storage_immutable(&shdw_drive_client, &v2_pubkey).await;

    // // 3.
    // add_immutable_storage_test(&shdw_drive_client, &v1_pubkey).await;
    add_immutable_storage_test(&shdw_drive_client, &v2_pubkey).await;
}

async fn create_storage_accounts<T: Signer>(shdw_drive_client: ShadowDriveClient<T>) {
    let result_v1 = shdw_drive_client
        .create_storage_account(
            "shdw-drive-1.5-test-v1",
            Byte::from_str("1MB").expect("invalid byte string"),
            StorageAccountVersion::v1(),
        )
        .await
        .expect("error creating storage account");

    let result_v2 = shdw_drive_client
        .create_storage_account(
            "shdw-drive-1.5-test-v2",
            Byte::from_str("1MB").expect("invalid byte string"),
            StorageAccountVersion::v2(),
        )
        .await
        .expect("error creating storage account");

    println!("v1: {:?}", result_v1);
    println!("v2: {:?}", result_v2);
}

async fn make_storage_immutable<T: Signer>(
    shdw_drive_client: &ShadowDriveClient<T>,
    storage_account_key: &Pubkey,
) {
    let storage_account = shdw_drive_client
        .get_storage_account(storage_account_key)
        .await
        .expect("failed to get storage account");
    match storage_account {
        StorageAcct::V1(storage_account) => println!("account: {:?}", storage_account),
        StorageAcct::V2(storage_account) => println!("account: {:?}", storage_account),
    }

    let make_immutable_response = shdw_drive_client
        .make_storage_immutable(&storage_account_key)
        .await
        .expect("failed to make storage immutable");

    println!("response: {:?}", make_immutable_response);

    let storage_account = shdw_drive_client
        .get_storage_account(&storage_account_key)
        .await
        .expect("failed to get storage account");
    match storage_account {
        StorageAcct::V1(storage_account) => println!("account: {:?}", storage_account),
        StorageAcct::V2(storage_account) => println!("account: {:?}", storage_account),
    }
}

async fn add_immutable_storage_test<T: Signer>(
    shdw_drive_client: &ShadowDriveClient<T>,
    storage_account_key: &Pubkey,
) {
    let storage_account = shdw_drive_client
        .get_storage_account(&storage_account_key)
        .await
        .expect("failed to get storage account");

    match storage_account {
        StorageAcct::V1(storage_account) => {
            println!("old size: {:?}", storage_account.reserved_bytes)
        }
        StorageAcct::V2(storage_account) => {
            println!("old size: {:?}", storage_account.reserved_bytes)
        }
    }

    let add_immutable_storage_response = shdw_drive_client
        .add_immutable_storage(
            storage_account_key,
            Byte::from_str("1MB").expect("invalid byte string"),
        )
        .await
        .expect("error adding storage");

    println!("response: {:?}", add_immutable_storage_response);

    let storage_account = shdw_drive_client
        .get_storage_account(&storage_account_key)
        .await
        .expect("failed to get storage account");

    match storage_account {
        StorageAcct::V1(storage_account) => {
            println!("new size: {:?}", storage_account.reserved_bytes)
        }
        StorageAcct::V2(storage_account) => {
            println!("new size: {:?}", storage_account.reserved_bytes)
        }
    }
}
