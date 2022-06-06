use serde_json::json;
use shadow_drive_rust::Client;
use solana_account_decoder::{UiAccount, UiAccountEncoding};
use solana_client::{
    rpc_client::RpcClient,
    rpc_request::RpcRequest,
    rpc_response::{Response, RpcResponseContext},
};
use solana_sdk::{account::Account, pubkey::Pubkey, signer::keypair::Keypair};
use std::collections::HashMap;
use std::str::FromStr;

#[tokio::test]
async fn test_get_storage_account() {
    let keypair = Keypair::new();
    let storage_account_key = Pubkey::new_unique();

    // Prepare mocks
    let mut mocks = HashMap::new();

    let mock_storage_account = Account {
        lamports: 4370880,
        owner: Pubkey::from_str("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ").unwrap(),
        executable: false,
        rent_epoch: 315,
        data: vec![
            41, 48, 231, 194, 22, 77, 205, 235, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            16, 0, 0, 0, 0, 0, 240, 255, 15, 0, 0, 0, 0, 0, 170, 45, 83, 70, 149, 224, 248, 249,
            205, 65, 226, 162, 74, 109, 121, 172, 189, 217, 49, 162, 128, 145, 131, 15, 4, 191,
            167, 237, 62, 83, 128, 244, 170, 45, 83, 70, 149, 224, 248, 249, 205, 65, 226, 162, 74,
            109, 121, 172, 189, 217, 49, 162, 128, 145, 131, 15, 4, 191, 167, 237, 62, 83, 128,
            244, 204, 5, 71, 107, 158, 92, 71, 205, 137, 76, 228, 154, 6, 152, 185, 167, 214, 215,
            174, 143, 172, 126, 167, 127, 86, 15, 10, 61, 25, 124, 48, 160, 19, 0, 0, 0, 0, 0, 16,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 201, 64, 154, 98, 59, 1, 0, 0, 59, 1, 0, 0, 7,
            0, 0, 0, 109, 121, 45, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
    };

    let encoded_mock_account = UiAccount::encode(
        &storage_account_key,
        &mock_storage_account,
        UiAccountEncoding::JsonParsed,
        None,
        None,
    );

    let get_storage_account_response = json!(Response {
        context: RpcResponseContext { slot: 1 },
        value: encoded_mock_account,
    });
    mocks.insert(RpcRequest::GetAccountInfo, get_storage_account_response);

    // Create RPC + Client
    let mock_rpc = RpcClient::new_mock_with_mocks("https://ssc-dao.genesysgo.net", mocks);
    let shdw_drive_client = Client::new(keypair, mock_rpc);

    // get account
    let storage_account = shdw_drive_client
        .get_storage_account(&storage_account_key)
        .await
        .expect("failed to get storage account");

    assert_eq!(storage_account.is_static, true);
    assert_eq!(storage_account.init_counter, 1);
    assert_eq!(storage_account.del_counter, 0);
    assert_eq!(storage_account.immutable, false);
    assert_eq!(storage_account.to_be_deleted, false);
    assert_eq!(storage_account.delete_request_epoch, 0);
    assert_eq!(storage_account.storage, 1048576);
    assert_eq!(storage_account.storage_available, 1048560);
    assert_eq!(
        storage_account.owner_1,
        Pubkey::from_str("CTJPtEeFGj1Tz5gsSKbfJhQFLnTwFTMqQu5LTG7Tc3vK").unwrap()
    );
    assert_eq!(
        storage_account.owner_2,
        Pubkey::from_str("CTJPtEeFGj1Tz5gsSKbfJhQFLnTwFTMqQu5LTG7Tc3vK").unwrap()
    );
    assert_eq!(
        storage_account.shdw_payer,
        Pubkey::from_str("EjQqfkVGpoahPqqMHGy8HW3hBgNgKBeLb7tSWJCngApo").unwrap()
    );
    assert_eq!(storage_account.account_counter_seed, 19);
    assert_eq!(storage_account.total_cost_of_current_storage, 1048576);
    assert_eq!(storage_account.total_fees_paid, 0);
    assert_eq!(storage_account.creation_time, 1654276297);
    assert_eq!(storage_account.creation_epoch, 315);
    assert_eq!(storage_account.last_fee_epoch, 315);
    assert_eq!(storage_account.identifier, "my-test");
}
