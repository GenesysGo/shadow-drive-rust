use solana_sdk::pubkey::Pubkey;

use crate::constants::PROGRAM_ADDRESS;

/// Returns the program derived address and bump seed for a [`StorageAccount`](crate::models::StorageAccount).
pub fn storage_account(wallet_pubkey: &Pubkey, account_seed: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &b"storage-account"[..],
            &wallet_pubkey.to_bytes(),
            &account_seed.to_le_bytes(),
        ],
        &PROGRAM_ADDRESS,
    )
}

/// Returns the program derived address and bump seed for a [`StorageAccount`](crate::models::StorageAccount)'s [`FileAccount`](crate::models::FileAccount).
pub fn file_account(storage_account: &Pubkey, file_seed: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&storage_account.to_bytes(), &file_seed.to_le_bytes()],
        &PROGRAM_ADDRESS,
    )
}

/// Returns the program derived address and bump seed for a wallet's [`UserInfo`](crate::models::UserInfo) account.
pub fn user_info(wallet_pubkey: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"user-info"[..], &wallet_pubkey.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

/// Returns the program derived address and bump seed for a [`StorageAccount`](crate::models::StorageAccount)'s stake account.
/// The stake account is a SHDW token account that holds user's stake.
pub fn stake_account(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"stake-account"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}
/// Returns the program derived address and bump seed for a [`StorageAccount`](crate::models::StorageAccount)'s stake account.
/// The unstake account is a token account that handles SHDW when unstaking.
pub fn unstake_account(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"unstake-account"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

/// Returns the program derived address and bump seed for an [`UnstakeInfo`](crate::models::UnstakeInfo).
pub fn unstake_info(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"unstake-info"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

pub fn migration_helper(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"migration-helper"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}
