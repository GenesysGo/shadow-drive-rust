use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2}, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};


/// This is the function that handles the `unmark_delete_account` ix
pub fn handler(mut ctx: impl UnmarkDeleteAccount) -> Result<()> {
    // Cannot request if marked as immutable
    require!(
        !ctx.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // Cannot unmark to delete if already unmarked
    require!(
        ctx.check_delete_flag(),
        ErrorCodes::AccountNotMarkedToBeDeleted
    );

    // Cannot unmark to delete if stake_balance is zero
    require_gt!(
        ctx.get_balance(),
        0,
        ErrorCodes::EmptyStakeAccount
    );

    msg!(
        "Unmarking storage account {} for deletion",
        ctx.get_identifier()
    );
    ctx.unmark_delete();

    Ok(())
}

#[derive(Accounts)]
/// This `UnmarkDeleteAccount` context is used in the instruction which allows users to
/// unmark an account for future deletion (by an admin).
pub struct UnmarkDeleteAccountV1<'info> {
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

    /// Stake account associated with storage account
    #[account(
        mut,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes()
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

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
/// This `UnmarkDeleteAccount` context is used in the instruction which allows users to
/// unmark an account for future deletion (by an admin).
pub struct UnmarkDeleteAccountV2<'info> {
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

    /// Stake account associated with storage account
    #[account(
        mut,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes()
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

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

type Shades = u64;

pub trait UnmarkDeleteAccount {
    fn check_immutable(&self) -> bool;
    fn check_delete_flag(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn get_balance(&self) -> Shades;
    fn unmark_delete(&mut self);
}

impl UnmarkDeleteAccount for Context<'_,'_,'_,'_, UnmarkDeleteAccountV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool {
        self.accounts.storage_account.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn get_balance(&self) -> Shades {
        self.accounts.stake_account.amount
    }
    fn unmark_delete(&mut self) {
        let storage_account = &mut self.accounts.storage_account;

        // Update deletion flag and reset request time
        storage_account.to_be_deleted = false;
        storage_account.delete_request_epoch = 0;
    }
}


impl UnmarkDeleteAccount for Context<'_,'_,'_,'_, UnmarkDeleteAccountV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool {
        self.accounts.storage_account.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn get_balance(&self) -> Shades {
        self.accounts.stake_account.amount
    }
    fn unmark_delete(&mut self) {
        let storage_account = &mut self.accounts.storage_account;

        // Update deletion flag and reset request time
        storage_account.to_be_deleted = false;
        storage_account.delete_request_epoch = 0;
    }
}
