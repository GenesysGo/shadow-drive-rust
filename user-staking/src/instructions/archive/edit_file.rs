use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::store_file::{is_owner, File};
use crate::instructions::{initialize_account::StorageAccount, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
// use std::convert::TryInto;

/// This is the function that handles the `edit_file` ix
pub fn handler(ctx: Context<EditFile>, size: u64, sha256_hash: String) -> Result<()> {
    // Cannot edit file if file is immutable
    require!(
        !ctx.accounts.file.immutable,
        ErrorCodes::FileMarkedImmutable
    );

    // Cannot edit file if storage account is immutable
    require!(
        !ctx.accounts.storage_account.immutable,
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // msg!("Editing File account: {}", filename);
    let old_size;
    {
        let file = &mut ctx.accounts.file;

        // Note: These comments are intentionally left here from the store_file ix to explicitly show what fields are not modified.

        // Store owner
        // file.owner = ctx.accounts.owner.key();

        // Store time file was created/stored
        // file.created = created;

        // Initialize as mutable
        // file.immutable = false;

        // Initialize deletion flag
        // file.to_be_deleted = false;

        // Initialize delete request time
        // file.delete_request_time = std::i64::MIN;

        // Store file size
        // Here, we must check that any positive change in size does not exceed storage_available. There is obviously no problem if the file shrinks.
        // NOTE: Now that we are not tracking storage on-chain in v1.5, this is the wrong condition,
        // as it should check storage_available > size. It is up to the uploader server to 
        // check this condition! For now, we do this minimal sanity check whether the file
        // is smaller than the total storage on-chain.
        require_gte!(
            ctx.accounts.storage_account.storage,
            size.saturating_sub(file.size),
            ErrorCodes::NotEnoughStorage
        );
        old_size = file.size;
        file.size = size;

        // Store sha256 hash
        file.store_sha256(&sha256_hash);
        // file.initial_sha256_hash = sha256_hash.clone();

        // Store file name
        // require!(
        //     filename.as_bytes().len() <= MAX_FILENAME_SIZE,
        //     ErrorCodes::FileNameLengthExceedsLimit
        // );
        // file.name = filename;

        // Store file URL
        // file.url = url;
    }

    // No longer doing this on-chain as of v1.5
    // msg!(
    //     "Modifying Parent Storage Account: {}",
    //     ctx.accounts.storage_account.identifier
    // );
    // {
    //     let storage_account = &mut ctx.accounts.storage_account;

    //     // Change storage available if file size changed
    //     storage_account.storage_available =
    //         validate_storage_change(storage_account.storage_available, old_size, size)?;
    // }

    Ok(())
}

#[derive(Accounts)]
/// This `EditFile` context is used in the instruction that allows users
/// to edit files, metadata.
pub struct EditFile<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    /// Requires mutability to update global storage counter.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account.
    /// Requires mutability to update user storage account storage counter.
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

    /// Child file account.
    /// Requires mutability since we are editing.
    #[account(
        mut,
        seeds = [
            &storage_account.key().to_bytes(),
            &file.init_counter_seed.to_le_bytes(),
        ],
        bump,
    )]
    pub file: Account<'info, File>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=is_owner(&owner, &storage_account))]
    pub owner: Signer<'info>,

    /// Uploader needs to sign to ensure all is well on storage server (incl CSAM scan).
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}

fn validate_storage_change(storage_available: u64, old_size: u64, size: u64) -> Result<u64> {
    // Case: file grows
    if size > old_size {
        let size_change = size.checked_sub(old_size).unwrap();

        let result = storage_available.checked_sub(size_change);
        if let Some(valid) = result {
            // Return new storage if there is enough space
            Ok(valid)
        } else {
            // Return Err if there is not enough space
            err!(ErrorCodes::NotEnoughStorage)
        }
    }
    // Case: file stays the same size or shrinks
    else {
        let size_change = old_size.checked_sub(size).unwrap();
        Ok(storage_available.checked_add(size_change).unwrap())
    }
}
