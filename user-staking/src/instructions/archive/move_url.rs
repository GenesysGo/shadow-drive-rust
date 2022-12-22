use crate::constants::*;
use crate::instructions::store_file::{is_owner, File};
use crate::instructions::{initialize_account::StorageAccount, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

/// This is the function that handles the `move_url` ix
pub fn handler(ctx: Context<MoveUrl>, url: String) -> Result<()> {
    msg!("Moving url of file: {}", ctx.accounts.file.name);
    {
        let file = &mut ctx.accounts.file;

        // Update url
        require!(
            url.as_bytes().len() <= MAX_URL_SIZE,
            MoveUrlError::UrlExceedsMaxSize
        );
        file.url = url;
    }
    Ok(())
}

#[derive(Accounts)]
/// This `MoveUrl` context is used in the instruction which allows an admin to move the url of a file.
pub struct MoveUrl<'info> {
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
    /// CHECK: This is fine because we have a constraint.
    #[account(mut, constraint=is_owner(&owner, &storage_account))]
    pub owner: AccountInfo<'info>,

    /// Uploader needs to sign to ensure all is well on storage server (incl CSAM scan).
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum MoveUrlError {
    #[msg("URL exceeds max byte size")]
    UrlExceedsMaxSize,
}
