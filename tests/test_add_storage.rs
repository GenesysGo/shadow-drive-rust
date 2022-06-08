use shadow_drive_rust::Client;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_sdk::signature::Signer;
use solana_sdk::{pubkey::Pubkey, signer::keypair::Keypair};
use std::collections::HashMap;
use byte_unit::Byte;

#[tokio::test]
async fn test_add_storage() {
    let keypair = Keypair::new();
    let storage_account_key = Pubkey::new_unique();

    // Prepare mocks
    let mut mocks = HashMap::new();
    let get_storage_account_response =
        test_utilities::basic_storage_account_response(keypair.pubkey(), storage_account_key);
    mocks.insert(RpcRequest::GetAccountInfo, get_storage_account_response);

    // Create RPC + Client
    let mock_rpc = RpcClient::new_mock_with_mocks("https://ssc-dao.genesysgo.net", mocks);
    let shdw_drive_client = Client::new(keypair, mock_rpc);

    // get account
    let add_storage_response = shdw_drive_client
        .add_storage(&storage_account_key,
            Byte::from_str("1MB").expect("invalid byte string"))
        .await
        .expect("failed to add storage");

    assert!(add_storage_response.txid.len() > 0);
}
