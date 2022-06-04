use solana_sdk::pubkey::Pubkey;

use crate::constants::PROGRAM_ADDRESS;

pub fn storage_account(key: &Pubkey, account_seed: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &b"storage-account"[..],
            &key.to_bytes(),
            &account_seed.to_le_bytes(),
        ],
        &PROGRAM_ADDRESS,
    )
}

pub fn file_account(storage_account: &Pubkey, file_seed: u32) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&storage_account.to_bytes(), &file_seed.to_le_bytes()],
        &PROGRAM_ADDRESS,
    )
}

pub fn user_info(wallet_pubkey: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"user-info"[..], &wallet_pubkey.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

pub fn stake_account(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"stake-account"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

pub fn unstake_account(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"unstake-account"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}

pub fn unstake_info(storage_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[&b"unstake-info"[..], &storage_account.to_bytes()],
        &PROGRAM_ADDRESS,
    )
}
