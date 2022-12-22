use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::store_file::{is_owner, File};
use crate::instructions::{initialize_account::StorageAccount, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::convert::TryInto;

/// This is the function that handles the `request_delete_file` ix
pub fn handler(ctx: Context<RequestDeleteFile>) -> Result<()> {
    // Cannot request to delete file if immutable
    require!(
        !ctx.accounts.file.immutable,
        ErrorCodes::FileMarkedImmutable
    );

    // Cannot mark to delete if already marked
    require!(
        !ctx.accounts.file.to_be_deleted,
        ErrorCodes::AlreadyMarkedForDeletion
    );

    msg!(
        "Requesting to delete File account: {}",
        ctx.accounts.file.name
    );
    {
        let file = &mut ctx.accounts.file;

        // Update deletion flag and record request time
        file.to_be_deleted = true;
        file.delete_request_epoch = Clock::get()?.epoch.try_into().unwrap();
    }

    Ok(())
}

#[derive(Accounts)]
/// This `RequestDeleteFile` context is used in the instruction which allows users to
/// mark a file for future deletion (by an admin).
pub struct RequestDeleteFile<'info> {
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
