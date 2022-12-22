use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::store_file::{is_owner, File};
use crate::instructions::{initialize_account::StorageAccount, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

/// This is the function that handles the `unmark_delete_file` ix
pub fn handler(ctx: Context<UnmarkDeleteFile>) -> Result<()> {
    // Cannot request if marked as immutable
    require!(
        !ctx.accounts.file.immutable,
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // Cannot unmark to delete if already unmarked
    require!(
        ctx.accounts.file.to_be_deleted,
        ErrorCodes::FileNotMarkedToBeDeleted
    );

    // Cannot unmark to delete if stake_balance is zero
    require!(
        ctx.accounts.stake_account.amount > 0,
        ErrorCodes::EmptyStakeAccount
    );

    msg!(
        "Unmarking file account {} for deletion",
        ctx.accounts.file.name
    );
    {
        let file = &mut ctx.accounts.file;

        // Update deletion flag and reset request time
        file.to_be_deleted = false;
        file.delete_request_epoch = 0;
    }

    Ok(())
}

#[derive(Accounts)]
/// This `UnmarkDeleteFile` context is used in the instruction which allows users to
/// unmark a file for future deletion (by an admin).
pub struct UnmarkDeleteFile<'info> {
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
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes()
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

    /// Child file account
    #[account(
        mut,
        seeds = [
            &storage_account.key().to_bytes(),
            &file.init_counter_seed.to_le_bytes()
        ],
        bump,
    )]
    pub file: Account<'info, File>,

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
    #[account(mut, constraint=is_owner(&owner, &storage_account))]
    pub owner: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}
