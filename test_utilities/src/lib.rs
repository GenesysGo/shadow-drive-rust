use anchor_lang::AccountSerialize;
use std::str::FromStr;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
};
use solana_client::{
    rpc_response::{Response, RpcResponseContext},
};
use shadow_drive_user_staking::instructions::initialize_account::StorageAccount;
use solana_account_decoder::{UiAccount, UiAccountEncoding};
use serde_json::json;

    pub fn mock_storage_account(owner_key: Pubkey, identifier: String) -> Account { // payer too?

        let storage_account = StorageAccount {
            is_static: true,
            init_counter: 1,
            del_counter: 0,
            immutable: false,
            to_be_deleted: false,
            delete_request_epoch: 0,
            storage: 1048576,
            storage_available: 1048560,
            owner_1: owner_key,
            owner_2: owner_key,
            shdw_payer: Pubkey::from_str("EjQqfkVGpoahPqqMHGy8HW3hBgNgKBeLb7tSWJCngApo").unwrap(),
            account_counter_seed: 19,
            total_cost_of_current_storage: 1048576,
            total_fees_paid: 0,
            creation_time: 1654276297,
            creation_epoch: 315,
            last_fee_epoch: 315,
            identifier,
        };

        let mut data: Vec<u8> = Vec::new();
        storage_account.try_serialize(&mut data).unwrap();

        Account {
            lamports: 4370880,
            owner: Pubkey::from_str("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ").unwrap(),
            executable: false,
            rent_epoch: 315,
            data,
        }
    }

    pub fn basic_storage_account_response(owner_key: Pubkey, storage_account_key: Pubkey) -> serde_json::Value {
        let mock_storage_account = mock_storage_account(owner_key, "first-test".to_string());

        let encoded_mock_account = UiAccount::encode(
            &storage_account_key,
            &mock_storage_account,
            UiAccountEncoding::JsonParsed,
            None,
            None,
        );

        json!(Response {
            context: RpcResponseContext { slot: 1 },
            value: encoded_mock_account,
        })
    }
