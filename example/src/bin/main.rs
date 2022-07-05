use byte_unit::Byte;
use shadow_drive_rust::{models::ShadowFile, ShadowDriveClient, StorageAccount};
use solana_sdk::{
    pubkey::Pubkey,
    signer::{keypair::read_keypair_file, Signer},
};

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    let pubkey = keypair.pubkey();
    let (storage_account_key, _) =
        shadow_drive_rust::derived_addresses::storage_account(&pubkey, 0);

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    get_storage_accounts_test(shdw_drive_client, &storage_account_key).await
}

async fn get_storage_accounts_test<T: Signer + Send + Sync>(
    shdw_drive_client: ShadowDriveClient<T>,
    pubkey: &Pubkey,
) {
    let storage_account = shdw_drive_client
        .get_storage_account(pubkey)
        .await
        .expect("failed to get storage account");
    println!(
        "identifier: {:?}; immutable: {:?}",
        storage_account.get_identifier(),
        storage_account.check_immutable()
    );
}

// async fn list_objects_test<T: Signer + Send + Sync>(
//     shdw_drive_client: ShadowDriveClient<T>,
//     storage_account_key: &Pubkey,
// ) {
//     let objects = shdw_drive_client
//         .list_objects(storage_account_key)
//         .await
//         .expect("failed to list objects");

//     println!("objects {:?}", objects);
// }

// async fn make_storage_immutable_test<T: Signer + Send + Sync>(
//     shdw_drive_client: ShadowDriveClient<T>,
//     storage_account_key: &Pubkey,
// ) {
//     let storage_account = shdw_drive_client
//         .get_storage_account(storage_account_key)
//         .await
//         .expect("failed to get storage account");
//     println!(
//         "identifier: {:?}; immutable: {:?}",
//         storage_account.identifier, storage_account.immutable
//     );

//     let make_immutable_response = shdw_drive_client
//         .make_storage_immutable(&storage_account_key)
//         .await
//         .expect("failed to make storage immutable");

//     println!("txn id: {:?}", make_immutable_response.txid);

//     let storage_account = shdw_drive_client
//         .get_storage_account(&storage_account_key)
//         .await
//         .expect("failed to get storage account");
//     println!(
//         "identifier: {:?}; immutable: {:?}",
//         storage_account.identifier, storage_account.immutable
//     );
// }

// async fn add_storage_test<T: Signer + Send + Sync>(
//     shdw_drive_client: &ShadowDriveClient<T>,
//     storage_account_key: &Pubkey,
// ) {
//     let storage_account = shdw_drive_client
//         .get_storage_account(&storage_account_key)
//         .await
//         .expect("failed to get storage account");

//     let add_storage_response = shdw_drive_client
//         .add_storage(
//             storage_account_key,
//             Byte::from_str("10MB").expect("invalid byte string"),
//         )
//         .await
//         .expect("error adding storage");

//     println!("txn id: {:?}", add_storage_response.txid);

//     let storage_account = shdw_drive_client
//         .get_storage_account(&storage_account_key)
//         .await
//         .expect("failed to get storage account");

//     println!("new size: {:?}", storage_account.storage);
// }

// async fn reduce_storage_test<T: Signer + Send + Sync>(
//     shdw_drive_client: ShadowDriveClient<T>,
//     storage_account_key: &Pubkey,
// ) {
//     let storage_account = shdw_drive_client
//         .get_storage_account(storage_account_key)
//         .await
//         .expect("failed to get storage account");

//     println!("previous size: {:?}", storage_account.storage);

//     let add_storage_response = shdw_drive_client
//         .reduce_storage(
//             storage_account_key,
//             Byte::from_str("10MB").expect("invalid byte string"),
//         )
//         .await
//         .expect("error adding storage");

//     println!("txn id: {:?}", add_storage_response.txid);

//     let storage_account = shdw_drive_client
//         .get_storage_account(storage_account_key)
//         .await
//         .expect("failed to get storage account");

//     println!("new size: {:?}", storage_account.storage);
// }

// async fn upload_file_test<T: Signer + Send + Sync>(
//     shdw_drive_client: ShadowDriveClient<T>,
//     storage_account_key: &Pubkey,
// ) {
//     let upload_reponse = shdw_drive_client
//         .upload_file(
//             storage_account_key,
//             ShadowFile::file(String::from("example.png"), "example.png"),
//         )
//         .await
//         .expect("failed to upload file");

//     println!("Upload complete {:?}", upload_reponse);
// }
