use anchor_lang::AccountSerialize;
use std::str::FromStr;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey
};
use shadow_drive_user_staking::instructions::initialize_account::StorageAccount;

    pub fn mock_storage_account_1() -> Account {

        let storage_account = StorageAccount {
            is_static: true,
            init_counter: 1,
            del_counter: 0,
            immutable: false,
            to_be_deleted: false,
            delete_request_epoch: 0,
            storage: 1048576,
            storage_available: 1048560,
            owner_1: Pubkey::from_str("CTJPtEeFGj1Tz5gsSKbfJhQFLnTwFTMqQu5LTG7Tc3vK").unwrap(),
            owner_2: Pubkey::from_str("CTJPtEeFGj1Tz5gsSKbfJhQFLnTwFTMqQu5LTG7Tc3vK").unwrap(),
            shdw_payer: Pubkey::from_str("EjQqfkVGpoahPqqMHGy8HW3hBgNgKBeLb7tSWJCngApo").unwrap(),
            account_counter_seed: 19,
            total_cost_of_current_storage: 1048576,
            total_fees_paid: 0,
            creation_time: 1654276297,
            creation_epoch: 315,
            last_fee_epoch: 315,
            identifier: "first-test".to_string()
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

    pub fn mock_storage_account_2() -> Account {

        let storage_account = StorageAccount {
            is_static: true,
            init_counter: 5,
            del_counter: 0,
            immutable: false,
            to_be_deleted: false,
            delete_request_epoch: 0,
            storage: 1048576,
            storage_available: 1048560,
            owner_1: Pubkey::from_str("2bqJYcA1A8gw4qJFjyE2G4akiUunpd9rP6QzfnxHqSqr").unwrap(),
            owner_2: Pubkey::from_str("2bqJYcA1A8gw4qJFjyE2G4akiUunpd9rP6QzfnxHqSqr").unwrap(),
            shdw_payer: Pubkey::from_str("HWxFaUAZmidATkd8ji91StGQYfL5SbgjPotAUNnMWGHt").unwrap(),
            account_counter_seed: 7,
            total_cost_of_current_storage: 1048576,
            total_fees_paid: 0,
            creation_time: 1654276297,
            creation_epoch: 200,
            last_fee_epoch: 200,
            identifier: "second-test".to_string()
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
