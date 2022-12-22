use crate::instructions::initialize_account::StorageAccount;
use anchor_lang::prelude::*;

/// This is the function that handles the `redeem_rent` ix
pub fn handler(ctx: Context<RedeemRent>) -> Result<()> {

    msg!("Deleting File {}", ctx.accounts.file.name);
    // account is marked with `close` in Context<RedeemRent>

    Ok(())
}

#[derive(Accounts)]
/// This `RedeemRent` context is used in the instruction that allows users
/// to delete file accounts.
pub struct RedeemRent<'info> {

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

    /// File account to be closed
    #[account(
        mut,
        close = owner,
        seeds = [
            &storage_account.key().to_bytes(),
            &file.init_counter_seed.to_le_bytes(),
        ],
        bump,
        constraint = file.storage_account == storage_account.key(), // a redundant constraint just to be sure
    )]
    pub file: Account<'info, File>,

    /// File owner, user
    #[account(mut, constraint=is_owner(&owner, &storage_account))]
    pub owner: Signer<'info>,
}

#[account]
pub struct File {
    /// Mutability
    pub immutable: bool,

    /// Delete flag
    pub to_be_deleted: bool,

    /// Delete request epoch
    pub delete_request_epoch: u32,

    /// File size (bytes)
    pub size: u64,

    /// File hash (sha256)
    pub sha256_hash: [u8; 32],

    /// File counter seed
    pub init_counter_seed: u32,

    /// Storage accout
    pub storage_account: Pubkey,

    /// File name
    pub name: String,
}

pub fn is_owner<'info>(
    owner: &AccountInfo<'info>,
    storage_account: &Account<'info, StorageAccount>,
) -> bool {
    owner.key() == storage_account.owner_1 || owner.key() == storage_account.owner_2
}
