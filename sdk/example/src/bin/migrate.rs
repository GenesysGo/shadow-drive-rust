use byte_unit::Byte;
use shadow_drive_sdk::{ShadowDriveClient, StorageAccountVersion};
use solana_sdk::{pubkey::Pubkey, signer::keypair::read_keypair_file};
use std::str::FromStr;

const KEYPAIR_PATH: &str = "keypair.json";

#[tokio::main]
async fn main() {
    //load keypair from file
    let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");

    //create shdw drive client
    let shdw_drive_client = ShadowDriveClient::new(keypair, "https://ssc-dao.genesysgo.net");

    // create V1 storage account
    let v1_response = shdw_drive_client
        .create_storage_account(
            "1.5-test",
            Byte::from_str("1MB").expect("invalid byte string"),
            StorageAccountVersion::v1(),
        )
        .await
        .expect("error creating storage account");

    println!("v1: {:?} \n", v1_response);

    let key_string: String = v1_response.shdw_bucket.unwrap();
    let v1_pubkey: Pubkey = Pubkey::from_str(&key_string).unwrap();

    // can migrate all at once
    let migrate = shdw_drive_client
        .migrate(&v1_pubkey)
        .await
        .expect("failed to migrate");
    println!("Migrated {:?} \n", migrate);

    // alternatively can split migration into 2 steps (boths steps are exposed)

    // // step 1
    // let migrate_step_1 = shdw_drive_client
    //     .migrate_step_1(&v1_pubkey)
    //     .await
    //     .expect("failed to migrate v1 step 1");
    // println!("Step 1 complete {:?} \n", migrate_step_1);

    // // step 2
    // let migrate_step_2 = shdw_drive_client
    //     .migrate_step_2(&v1_pubkey)
    //     .await
    //     .expect("failed to migrate v1 step 2");
    // println!("Step 2 complete {:?} \n", migrate_step_2);
}
