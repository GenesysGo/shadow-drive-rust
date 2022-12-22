use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2}, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::convert::TryInto;

/// This is the function that handles the `request_delete_account` ix
pub fn handler(mut ctx: impl RequestDeleteAccount) -> Result<()> {
    // Cannot request to delete storage account if immutable
    require!(
        !ctx.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // Cannot flag for deletion when already flagged for deletion
    require!(
        !ctx.check_delete_flag(),
        ErrorCodes::AlreadyMarkedForDeletion
    );

    msg!(
        "Requesting to delete StorageAccount account: {}",
        ctx.get_identifier()
    );
    ctx.mark_delete();

    Ok(())
}

#[derive(Accounts)]
/// This `RequestDeleteAccount` context is used in the instruction which allows users to
/// mark an account for future deletion (by an admin).
pub struct RequestDeleteAccountV1<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account.
    #[account(
        mut,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes()
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
/// This `RequestDeleteAccount` context is used in the instruction which allows users to
/// mark an account for future deletion (by an admin).
pub struct RequestDeleteAccountV2<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account.
    #[account(
        mut,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes()
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccountV2>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}

pub trait RequestDeleteAccount {
    fn check_immutable(&self) -> bool;
    fn check_delete_flag(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn mark_delete(&mut self);
}

impl RequestDeleteAccount for Context<'_,'_,'_,'_, RequestDeleteAccountV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool {
        self.accounts.storage_account.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn mark_delete(&mut self) {
        let storage_account = &mut self.accounts.storage_account;

        // Update deletion flag and record request time
        storage_account.to_be_deleted = true;
        storage_account.delete_request_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
}


impl RequestDeleteAccount for Context<'_,'_,'_,'_, RequestDeleteAccountV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool {
        self.accounts.storage_account.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn mark_delete(&mut self) {
        let storage_account = &mut self.accounts.storage_account;

        // Update deletion flag and record request time
        storage_account.to_be_deleted = true;
        storage_account.delete_request_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
}
