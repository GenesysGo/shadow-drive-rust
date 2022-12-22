use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    initialize_account::{StorageAccount, UserInfo},
    initialize_config::StorageConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use std::convert::TryInto;

// Discriminator + seed u64 + time i64 + booleans + size u64 + sha256 + txhash + name + url
pub const FILE_ACCOUNT_SIZE: usize = 8 // discriminator
    + 4 // delete request epoch
    + 8 // size
    + 1 // bools
    + 4 // seed
    + SHA256_HASH_SIZE
    + 4 + MAX_FILENAME_SIZE
    + 32; // storage_account pubkey
          // pub fn file_account_size(
          //     filename: &String
          // ) -> usize
          // {
          //     8 // discriminator
          //     + 4 // seed u32
          //     + 4 // delete_request_epoch u64
          //     + 1 // booleans
          //     + 8 // file size
          //     + SHA256_HASH_SIZE
          //     + filename.as_bytes().len()
          // }
          //    + MAX_URL_SIZE;

/// This is the function that handles the `store_file` ix
pub fn handler(
    ctx: Context<StoreFile>,
    filename: String,
    //url: String,
    size: u64,
    // created: i64,
    sha256_hash: String,
) -> Result<()> {
    // Ensure this user has never had a bad_csam
    require!(
        !ctx.accounts.user_info.lifetime_bad_csam,
        ErrorCodes::HasHadBadCsam
    );

    // Ensure account is not immutable
    require!(
        !ctx.accounts.storage_account.immutable,
        ErrorCodes::StorageAccountMarkedImmutable
    );

    msg!("Initializing child File account: {}", filename);
    {
        let file = &mut ctx.accounts.file;

        // Initialize as mutable
        file.immutable = false;

        // Initialize deletion flag
        file.to_be_deleted = false;

        // Initialize delete request time
        file.delete_request_epoch = Clock::get()?.epoch.try_into().unwrap();

        // Store file size
        // NOTE: Now that we are not tracking storage on-chain in v1.5, this is the wrong condition,
        // as it should check storage_available > size. It is up to the uploader server to 
        // check this condition! For now, we do this minimal sanity check whether the file
        // is smaller than the total storage on-chain.
        require_gte!(
            ctx.accounts.storage_account.storage,
            size,
            ErrorCodes::NotEnoughStorage
        );
        file.size = size;

        // Store sha256 hash
        file.store_sha256(&sha256_hash);

        // Store storage account
        file.storage_account = ctx.accounts.storage_account.key();

        // Store and increment file counter seed
        file.init_counter_seed = ctx.accounts.storage_account.init_counter;
        ctx.accounts.storage_account.increment_init_counter();

        // Store file name
        require!(
            filename.as_bytes().len() <= MAX_FILENAME_SIZE,
            ErrorCodes::FileNameLengthExceedsLimit
        );
        file.name = filename;

        // Store file URL
        //file.url = url;
    }

    // No longer done on chain as of v1.5
    // msg!(
    //     "Updating storage on parent StorageAccount: {}",
    //     ctx.accounts.storage_account.identifier
    // );
    // {
    //     let storage_account = &mut ctx.accounts.storage_account;

    //     // Decrease storage available
    //     storage_account.storage_available =
    //         validate_storage_available_sub(storage_account.storage_available, size)?;
    // }

    Ok(())
}

#[derive(Accounts)]
#[instruction(filename: String)]
/// This `StoreFile` context is used in the instruction which allows users to
/// store a file after our uploader server keypair signs off, verifying all is well.
pub struct StoreFile<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin and uploader keys.
    /// Requires mutability to update global storage counter.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account, which should already be initialized.
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

    // Child file account, to be initialized.
    #[account(
        init,
        payer = owner,
        space = FILE_ACCOUNT_SIZE,
        seeds = [
            storage_account.key().to_bytes().as_ref(),
            &storage_account.init_counter.to_le_bytes(),
        ],
        bump,
    )]
    pub file: Account<'info, File>,

    // Account containing user info
    #[account(
        mut,
        seeds = [
            "user-info".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
        ],
        bump,
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// File owner (user).
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint = is_owner(&owner, &storage_account))]
    pub owner: Signer<'info>,

    /// Uploader needs to sign to ensure all is well on storage server (incl CSAM scan).
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token mint account
    #[account(mut, address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
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
    // /// File url
    // pub url: String,
}

impl File {
    /// This function takes in a reference to a string or &str and turns it into a more compact byte array.
    /// A sha256 hash only needs 32 bytes to be stored, but takes up 64 bytes when stored with utf-8.
    pub fn store_sha256(&mut self, hash_string: &String) {
        let string_bytes = hex::decode(hash_string);
        self.sha256_hash = {
            if string_bytes.is_ok() {
                let string_bytes = string_bytes.unwrap();
                if string_bytes.len() == SHA256_HASH_SIZE {
                    Ok(string_bytes.try_into().unwrap())
                } else {
                    err!(ErrorCodes::InvalidSha256Hash).into()
                }
            } else {
                err!(ErrorCodes::InvalidSha256Hash).into()
            }
        }
        .unwrap();
    }

    // /// This function takes in a reference to a string or &str and turns it into a more compact byte array.
    // pub fn store_ceph(&mut self, hash_string: &String) {
    //     let string_bytes = hex::decode(hash_string);
    //     self.ceph_hash = {
    //         if string_bytes.is_ok() {
    //             let string_bytes = string_bytes.unwrap();
    //             if string_bytes.len() == CEPH_HASH_SIZE {
    //                 Ok(string_bytes.try_into().unwrap())
    //             } else {
    //                 err!(ErrorCodes::InvalidCEPHHash).into()
    //             }
    //         } else {
    //             err!(ErrorCodes::InvalidCEPHHash).into()
    //         }
    //     }
    //     .unwrap();
    // }
}

pub fn validate_storage_available_sub(storage_available: u64, size: u64) -> Result<u64> {
    let result = storage_available.checked_sub(size);
    if let Some(valid) = result {
        Ok(valid)
    } else {
        err!(ErrorCodes::NotEnoughStorage)
    }
}

pub fn is_owner<'info>(
    owner: &AccountInfo<'info>,
    storage_account: &Account<'info, StorageAccount>,
) -> bool {
    owner.key() == storage_account.owner_1 || owner.key() == storage_account.owner_2
    // || owner.key() == storage_account.owner_3
    // || owner.key() == storage_account.owner_4
}
