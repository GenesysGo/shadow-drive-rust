use serde_json::json;
use shadow_drive_rust::Client;
use solana_account_decoder::{UiAccount, UiAccountEncoding};
use solana_client::rpc_response::RpcKeyedAccount;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_sdk::signature::Signer;
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair};
use std::collections::HashMap;
use std::str::FromStr;

extern crate test_utilities;

#[tokio::test]
async fn test_get_storage_account() {
    let keypair = Keypair::new();
    let owner_pubkey = keypair.pubkey();
    let storage_account_key = Pubkey::new_unique();

    // Prepare mocks
    let mut mocks = HashMap::new();
    let get_storage_account_response =
        test_utilities::basic_storage_account_response(owner_pubkey, storage_account_key);
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
    assert_eq!(storage_account.owner_1, owner_pubkey);
    assert_eq!(storage_account.owner_2, owner_pubkey);
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
    assert_eq!(storage_account.identifier, "first-test");
}

#[tokio::test]
async fn test_get_multiple_storage_accounts() {
    let keypair = Keypair::new();
    let storage_account_key_1 = Pubkey::new_unique();
    let storage_account_key_2 = Pubkey::new_unique();
    let owner_key = Pubkey::new_unique();

    // Prepare mocks
    let mut mocks = HashMap::new();

    let mock_storage_account_1 =
        test_utilities::mock_storage_account(keypair.pubkey(), "first-test".to_string());
    let mock_storage_account_2 =
        test_utilities::mock_storage_account(keypair.pubkey(), "second-test".to_string());

    let encoded_storage_1 = UiAccount::encode(
        &storage_account_key_1,
        &mock_storage_account_1,
        UiAccountEncoding::JsonParsed,
        None,
        None,
    );

    let encoded_storage_2 = UiAccount::encode(
        &storage_account_key_2,
        &mock_storage_account_2,
        UiAccountEncoding::JsonParsed,
        None,
        None,
    );

    let gpa_response = json!(vec![
        RpcKeyedAccount {
            pubkey: storage_account_key_1.to_string(),
            account: encoded_storage_1
        },
        RpcKeyedAccount {
            pubkey: storage_account_key_2.to_string(),
            account: encoded_storage_2
        }
    ]);
    mocks.insert(RpcRequest::GetProgramAccounts, gpa_response);

    // Create RPC + Client
    let mock_rpc = RpcClient::new_mock_with_mocks("https://ssc-dao.genesysgo.net", mocks);
    let shdw_drive_client = Client::new(keypair, mock_rpc);

    // get account
    let storage_accounts = shdw_drive_client
        .get_storage_accounts(&owner_key)
        .await
        .expect("failed to get storage account");

    assert_eq!(storage_accounts.len(), 2);
    assert_eq!(storage_accounts[0].identifier, "first-test");
    assert_eq!(storage_accounts[1].identifier, "second-test");
    assert_eq!(storage_accounts[0].account_counter_seed, 19);
    assert_eq!(storage_accounts[1].account_counter_seed, 19);
}
