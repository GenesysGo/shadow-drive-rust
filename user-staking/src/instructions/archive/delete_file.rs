use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::store_file::{is_owner, File};
use crate::instructions::{initialize_account::StorageAccount, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use std::convert::{TryFrom, TryInto};

/// This is the function that handles the `delete_file` ix
pub fn handler(ctx: Context<DeleteFile>) -> Result<()> {
    // Cannot request to delete storage account if immutable
    require!(
        !ctx.accounts.storage_account.immutable,
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // Cannot delete a file account not marked for deletion
    require!(
        ctx.accounts.file.to_be_deleted || ctx.accounts.storage_account.to_be_deleted,
        ErrorCodes::FileNotMarkedToBeDeleted
    );

    // Cannot delete a file account in grace period
    let clock = Clock::get()?;
    if ctx.accounts.file.to_be_deleted {
        require_gte!(
            u32::try_from(clock.epoch).unwrap(),
            ctx.accounts
                .file
                .delete_request_epoch
                .checked_add(DELETION_GRACE_PERIOD as u32)
                .unwrap(),
            ErrorCodes::AccountStillInGracePeriod
        )
    }

    // Cannot delete a file account when storage account is in grace period
    if ctx.accounts.storage_account.to_be_deleted {
        require_gte!(
            u32::try_from(clock.epoch).unwrap(),
            ctx.accounts
                .storage_account
                .delete_request_epoch
                .checked_add(DELETION_GRACE_PERIOD as u32)
                .unwrap(),
            ErrorCodes::AccountStillInGracePeriod
        )
    }

    msg!("Deleting File {}", ctx.accounts.file.name);
    {
        // account is marked with `close` in Context<DeleteFile>

        // Increment del_counter
        ctx.accounts.storage_account.increment_del_counter();
    }

    msg!(
        "Updating storage on StorageAccount: {}",
        ctx.accounts.storage_account.identifier
    );
    {
        let storage_account = &mut ctx.accounts.storage_account;

        // Update storage account storage available
        storage_account.storage_available = storage_account
            .storage_available
            .checked_add(ctx.accounts.file.size.try_into().unwrap())
            .unwrap();
    }

    Ok(())
}

#[derive(Accounts)]
/// This `DeleteFile` context is used in the instruction that allows admins
/// to delete user file accounts.
pub struct DeleteFile<'info> {
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

    /// File account to be closed
    #[account(
        mut,
        close = owner,
        seeds = [
            &storage_account.key().to_bytes(),
            &file.init_counter_seed.to_le_bytes(),
        ],
        bump,
    )]
    pub file: Account<'info, File>,

    /// File owner, user
    /// CHECK: There is a constraint that checks whether this account is an owner.
    /// Also, our uploader keys are signing this transaction so presuamably we would only provide a good key.
    /// We also may not need this account at all.
    #[account(mut, constraint=is_owner(&owner, &storage_account))]
    pub owner: AccountInfo<'info>,

    /// Admin/uploader
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,
}
