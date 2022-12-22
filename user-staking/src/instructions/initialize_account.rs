use crate::{constants::*, errors::ErrorCodes, instructions::initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::convert::TryInto;

// pub const STORAGE_ACCOUNT_SIZE: usize = 
//    18 // Alignments (NOTE: THIS IS WRONG AND I ABANDONED IT.)
//  + 1 // bools (idk why we need ≈1 byte per)
//  + 2*4 // init and del counters u32
//  + 4 // delete request epoch u32
//  + 8*2 // storage, storage_available u64
//  + 32*3 // owner 1-4 + shdw payer pubkeys
//  + 4 // seed u32
//  + 8*2 // cost, fees u64
//  + 3*4 // creation time, epoch u32, last_fee epoch
//  + 4 + MAX_IDENTIFIER_SIZE; // identifier size, in bytes,

pub(crate) fn calc_v1_storage(identifier: &str) -> usize {
    std::mem::size_of::<StorageAccount>()
        .checked_add(identifier.as_bytes().len())
        .unwrap()
}

/// This is the function that handles the `initialize_account` ix
pub fn handler(
    mut ctx: impl InitializeStorageAccount,
    identifier: String,
    storage: u64,
    owner_2: Option<Pubkey>,
) -> Result<()> {


    // Initialize user_info if needed
    if ctx.get_account_counter() == 0 {
        msg!("Initializing UserInfo");
        ctx.initialize_user_info()?;
    }

    msg!("Initializing StorageAccount: {}", identifier);
    {
        require!(
            !ctx.check_csam(),
            ErrorCodes::HasHadBadCsam
        );

        // Store unique identifier.
        ctx.set_identifier(validate_identifier(identifier)?)?;

        // Initialize account-wide mutability flag
        let is_immutable = false;
        ctx.set_immutable(is_immutable)?;

        // Initialize deletion variables
        let to_be_deleted = false;
        let delete_request_epoch = 0;
        ctx.set_deletion_flag(to_be_deleted, delete_request_epoch)?;

        // Set local storage. Validated in the method
        ctx.change_storage(storage, Mode::Initialize)?;

        // Initialize file counters, account counter
        ctx.set_account_counter_seed()?;

        // Populate owners
        ctx.set_owner()?;
        ctx.set_owner2(owner_2)?;

        // Store time of creation
        ctx.record_genesis()?;
    }

    msg!("Staking user funds");
    {

        // Compute required stake to store data
        let shades_per_gib = ctx.get_shades_per_gib().unwrap();
        let stake_required_to_store: u64 = stake_required(storage, shades_per_gib);

        msg!(
            "User requires {} shades to store {} bytes",
            stake_required_to_store,
            storage
        );

        // Ensure user has enough funds
        let user_token_balance = ctx.get_user_token_balance();
        if user_token_balance.unwrap() < stake_required_to_store {
            return err!(ErrorCodes::InsufficientFunds);
        }

        // Transfer funds to stake_account
        ctx.stake_shades(stake_required_to_store)?;
    }

    msg!("Updating global storage on StorageConfig account");
    {

        // Decrease storage available
        ctx.change_global_storage(storage, Mode::Decrement)?;
    }

    msg!("Incrementing counter in UserInfo");
    ctx.increment_account_counter()?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(identifier: String)]
/// This `InitializeStorageAccount` context is used to initialize a `StorageAccount` which stores a user's
/// storage information including stake token account address, storage requested, and access keys.
pub struct InitializeStorageAccountV1<'info> {
    /// This account is a PDA that holds the storage configuration, including current cost per byte,
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// This account is a PDA that holds a user's info (not specific to one storage account).
    #[account(
        init_if_needed,
        payer = owner_1,
        space = {
            8 // discriminator
            + 4 // init counter
            + 4 // del counter
            + 2 // bools
        },
        seeds = [
            "user-info".as_bytes(),
            &owner_1.key().to_bytes(),
        ],
        bump,
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// This account is a PDA that holds a user's `StorageAccount` information.
    #[account(
        init,
        payer = owner_1,
        space = calc_v1_storage(&identifier),
        seeds = [
            "storage-account".as_bytes(),
            &owner_1.key().to_bytes(),
            &user_info.account_counter.to_le_bytes(),
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

    /// This token account serves as the account which holds user's stake for file storage.
    #[account(
        init,
        payer = owner_1,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = storage_config,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// This is the token in question for staking.
    #[account(address=crate::constants::shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// This is the user who is initializing the storage account
    /// and is automatically added as an admin
    #[account(mut)]
    pub owner_1: Signer<'info>,

    /// Uploader needs to sign as this txn
    /// needs to be fulfilled on the middleman server
    /// to create the ceph bucket
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_1_token_account.mint == shdw::ID)]
    pub owner_1_token_account: Box<Account<'info, TokenAccount>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
#[instruction(identifier: String)]
/// This `InitializeStorageAccount` context is used to initialize a `StorageAccount` which stores a user's
/// storage information including stake token account address, storage requested, and access keys.
pub struct InitializeStorageAccountV2<'info> {
    /// This account is a PDA that holds the storage configuration, including current cost per byte,
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// This account is a PDA that holds a user's info (not specific to one storage account).
    #[account(
        init_if_needed,
        payer = owner_1,
        space = {
            8 // discriminator
            + 4 // init counter
            + 4 // del counter
            + 2 // bools
        },
        seeds = [
            "user-info".as_bytes(),
            &owner_1.key().to_bytes(),
        ],
        bump,
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// This account is a PDA that holds a user's storage account information.
    /// Upgraded to `StorageAccountV2`.
    #[account(
        init,
        payer = owner_1,
        space = calc_v2_storage(&identifier),
        seeds = [
            "storage-account".as_bytes(),
            &owner_1.key().to_bytes(),
            &user_info.account_counter.to_le_bytes(),
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccountV2>>,

    /// This token account serves as the account which holds user's stake for file storage.
    #[account(
        init,
        payer = owner_1,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = storage_config,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// This is the token in question for staking.
    #[account(address=crate::constants::shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// This is the user who is initializing the storage account
    /// and is automatically added as an admin
    #[account(mut)]
    pub owner_1: Signer<'info>,

    /// Uploader needs to sign as this txn
    /// needs to be fulfilled on the middleman server
    /// to create the ceph bucket
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_1_token_account.mint == shdw::ID)]
    pub owner_1_token_account: Box<Account<'info, TokenAccount>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct StorageAccount {
    /// Immutable boolean to track what kind of storage account this is.
    /// NOTE: Not used in current implementation w/ non-dynamic storage payments
    pub is_static: bool,

    /// Flag on whether storage account is public (usable by anyone)
    //pub is_public: bool,

    /// Counter tracking how many files have been initialized
    pub init_counter: u32,

    /// Counter tracking how many files have been deleted
    pub del_counter: u32,

    /// Boolean to track whether storage account (and all child File accounts) are immutable
    pub immutable: bool,

    /// Delete flag
    pub to_be_deleted: bool,

    /// Delete request epoch
    pub delete_request_epoch: u32,

    /// Number of bytes of storage associated with this account
    pub storage: u64,

    /// Bytes available for use
    pub storage_available: u64,

    /// Primary owner of StorageAccount (immutable)
    pub owner_1: Pubkey,

    /// Optional owner 2
    pub owner_2: Pubkey,

    // /// Optional owner 3
    // pub owner_3: Pubkey,

    // /// Optional owner 4
    // pub owner_4: Pubkey,
    /// Pubkey of the token account that staked SHDW
    pub shdw_payer: Pubkey,

    /// Counter at time of initialization
    pub account_counter_seed: u32,

    /// Total shades paid for current box size
    pub total_cost_of_current_storage: u64,

    // Total shades paid for current box size
    pub total_fees_paid: u64,

    /// Time of storage account creation
    pub creation_time: u32,

    /// Time of storage account creation
    pub creation_epoch: u32,

    /// The last epoch through which the user paid
    pub last_fee_epoch: u32,

    /// Some unique identifier that the user provides.
    /// Serves as a seed for storage account PDA.
    pub identifier: String,
}


// pub const STORAGE_ACCOUNT_V2_SIZE: usize = 8 // discriminator
// + 1*2 // bools
// + 4 // delete request epoch u32
// + 8 // storage u64
// + 32*1 // owner 1 pubkey
// + 4 // seed u32
// + 8*1 // fees u64
// + 4*3 // creation time, epoch u32, last_fee epoch
// + 4 + MAX_IDENTIFIER_SIZE; // identifier size, in bytes,

pub(crate) fn calc_v2_storage(identifier: &str) -> usize {
    // STORAGE_ACCOUNT_V2_SIZE
    //     .checked_sub(MAX_IDENTIFIER_SIZE)
    //     .unwrap()
    //     .checked_add(identifier.as_bytes().len())
    //     .unwrap()
    std::mem::size_of::<StorageAccountV2>()
        .checked_add(identifier.as_bytes().len())
        .unwrap()
}


// #[test]
// fn print_size_of_v2() {
//     println!("V1: {STORAGE_ACCOUNT_SIZE}");
//     println!("V2: {STORAGE_ACCOUNT_V2_SIZE}");
// }

#[account]
pub struct StorageAccountV2 {
    // /// Immutable boolean to track what kind of storage account this is.
    // pub is_static: bool,

    // /// Flag on whether storage account is public (usable by anyone)
    // pub is_public: bool,

    // /// Counter tracking how many files have been initialized
    // pub init_counter: u32,

    // /// Counter tracking how many files have been deleted
    // pub del_counter: u32,

    /// Boolean to track whether storage account (and all child File accounts) are immutable
    pub immutable: bool,

    /// Delete flag
    pub to_be_deleted: bool,

    /// Delete request epoch
    pub delete_request_epoch: u32,

    /// Number of bytes of storage associated with this account
    pub storage: u64,

    // /// Bytes available for use
    // pub storage_available: u64,

    /// Primary owner of StorageAccount (immutable)
    pub owner_1: Pubkey,

    // /// Optional owner 2
    // pub owner_2: Pubkey,

    // /// Optional owner 3
    // pub owner_3: Pubkey,

    // /// Optional owner 4
    // pub owner_4: Pubkey,
    /// Pubkey of the token account that staked SHDW
    // pub shdw_payer: Pubkey,

    /// Counter at time of initialization
    pub account_counter_seed: u32,

    // /// Total shades paid for current box size
    // pub total_cost_of_current_storage: u64,

    // Total shades paid for current box size
    // pub total_fees_paid: u64,

    /// Time of storage account creation
    pub creation_time: u32,

    /// Time of storage account creation
    pub creation_epoch: u32,

    /// The last epoch through which the user paid
    pub last_fee_epoch: u32,

    /// Some unique identifier that the user provides.
    /// Serves as a seed for storage account PDA.
    pub identifier: String,
}

#[account]
pub struct UserInfo {
    /// Total number of storage accounts the user has with us
    pub account_counter: u32,

    /// Total number of storage accounts that have been deleted
    pub del_counter: u32,

    /// Boolean denoting that the user agreed to terms of service
    pub agreed_to_tos: bool,

    /// Boolean denoting whether this pubkey has ever had a bad scam scan
    pub lifetime_bad_csam: bool,
}

fn validate_identifier(identifier: String) -> Result<String> {
    if identifier.as_bytes().len() <= MAX_IDENTIFIER_SIZE {
        Ok(identifier)
    } else {
        err!(ErrorCodes::ExceededStorageLimit).into()
    }
}

fn stake_required(storage: u64, shades_per_gib: u64) -> u64 {
    // ((u128 * u128) / u128) --> u64 allows us to multiply u64's without overflow.
    // Should fail a lot less with nonzero inputs, even with std::u64::MAX.
    let result_u128: Option<u128> = (storage as u128)
        .checked_mul(shades_per_gib as u128)
        .unwrap()
        .checked_div(BYTES_PER_GIB as u128);
    result_u128.unwrap().try_into().unwrap()
}

type Storage = u64;
type Epoch = u32;

pub trait ShadowDriveStorageAccount {
    fn check_immutable(&self) -> bool;
    fn check_delete_flag(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn get_storage(&self) -> Storage;
    fn get_last_fee_epoch(&self) -> Epoch;
    fn mark_to_delete(&mut self);
    fn update_last_fee_epoch(&mut self);
    fn is_owner(&self, owner: Pubkey) -> bool;
}

impl ShadowDriveStorageAccount for StorageAccount {
    fn check_immutable(&self) -> bool {
        self.immutable
    }
    fn check_delete_flag(&self) -> bool {
        self.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.identifier.clone()
    }
    fn get_storage(&self) -> Storage {
        self.storage
    }
    fn get_last_fee_epoch(&self) -> Epoch {
        self.last_fee_epoch
    }
    fn mark_to_delete(&mut self) {
        self.to_be_deleted = true;
        self.delete_request_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
    fn update_last_fee_epoch(&mut self) {
        self.last_fee_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
    fn is_owner(&self, owner: Pubkey) -> bool {
        owner == self.owner_1 || owner == self.owner_2
    }
}

impl ShadowDriveStorageAccount for StorageAccountV2 {
    fn check_immutable(&self) -> bool {
        self.immutable
    }
    fn check_delete_flag(&self) -> bool {
        self.to_be_deleted
    }
    fn get_identifier(&self) -> String {
        self.identifier.clone()
    }
    fn get_storage(&self) -> Storage {
        self.storage
    }
    fn get_last_fee_epoch(&self) -> Epoch {
        self.last_fee_epoch
    }
    fn mark_to_delete(&mut self) {
        self.to_be_deleted = true;
        self.delete_request_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
    fn update_last_fee_epoch(&mut self) {
        self.last_fee_epoch = Clock::get().unwrap().epoch.try_into().unwrap();
    }
    fn is_owner(&self, owner: Pubkey) -> bool {
        owner == self.owner_1
    }
}


pub trait InitializeStorageAccount {
    fn set_immutable(&mut self, boolean: bool) -> Result<()>;
    fn set_identifier(&mut self, identifier: String) -> Result<()>;
    fn set_deletion_flag(&mut self, boolean: bool, epoch: u64) -> Result<()>;
    fn set_account_counter_seed(&mut self) -> Result<()>; // to be run once only
    fn change_storage(&mut self, bytes: u64, mode: Mode) -> Result<()>;
    fn set_owner(&mut self) -> Result<()>; // to be run once only
    fn set_owner2(&mut self, owner_2: Option<Pubkey>) -> Result<()>;
    fn record_genesis(&mut self) -> Result<()>; // to be run once only
    fn get_account_counter(&mut self) -> u32;
    fn initialize_user_info(&mut self) -> Result<()>; // to be run once only
    fn check_csam(&mut self) -> bool;
    fn change_global_storage(&mut self, storage: u64, mode: Mode) -> Result<()>;
    fn get_shades_per_gib(&mut self) -> Option<u64>;
    fn get_user_token_balance(&mut self) -> Option<u64>;
    fn stake_shades(&mut self, shades: u64) -> Result<()>;
    fn increment_account_counter(&mut self) -> Result<()>;
}

pub enum Mode {
    Increment,
    Decrement,
    Initialize,
}

impl InitializeStorageAccount for Context<'_, '_, '_, '_, InitializeStorageAccountV1<'_>> {
    fn set_immutable(&mut self, boolean: bool) -> Result<()> {
        self.accounts.storage_account.immutable = boolean;
        Ok(())
    }
    fn set_identifier(&mut self, identifier: String ) -> Result<()> {
        self.accounts.storage_account.identifier = identifier;
        Ok(())
    }
    fn set_deletion_flag(&mut self, boolean: bool, epoch: u64) -> Result<()> {
        self.accounts.storage_account.to_be_deleted = boolean;
        self.accounts.storage_account.delete_request_epoch = epoch.try_into().unwrap();
        Ok(())
    }
    fn set_account_counter_seed(&mut self) -> Result<()> {
        self.accounts.storage_account.account_counter_seed = self.get_account_counter();
        Ok(())
    }
    fn change_storage(&mut self, bytes: u64, mode: Mode) -> Result<()> {
        let bytes = self.accounts.storage_config.validate_storage(bytes)?;
        match mode {
            Mode::Increment => {
                self.accounts.storage_account.storage = self.accounts.storage_account.storage.checked_add(bytes).unwrap();
            },
            Mode::Decrement => {
                // The requirement should be that the bytes is less than what is available
                // but we are planning to no longer tracking this on-chain as of Jun 14.
                // As such, the uploader should sign off on this.
                self.accounts.storage_account.storage = self.accounts.storage_account.storage.checked_sub(bytes).unwrap();
            },
            Mode::Initialize => {
                self.accounts.storage_account.storage = bytes;
            }
        }
        Ok(())
    }
    fn set_owner(&mut self) -> Result<()> {
        self.accounts.storage_account.owner_1 = self.accounts.owner_1.key();
        Ok(())
    }
    fn set_owner2(&mut self, owner_2: Option<Pubkey>) -> Result<()>{
        if let Some(owner_2) = owner_2 {
            self.accounts.storage_account.owner_2 = owner_2;
        } 
        Ok(())
    }
    fn record_genesis(&mut self) -> Result<()> {
        let clock = Clock::get().unwrap();
        self.accounts.storage_account.creation_time = clock.unix_timestamp.try_into().unwrap();
        self.accounts.storage_account.creation_epoch = clock.epoch.try_into().unwrap();
        self.accounts.storage_account.last_fee_epoch = clock.epoch.try_into().unwrap();
        Ok(())
    }
    fn get_account_counter(&mut self) -> u32 {
        self.accounts.user_info.account_counter
    }
    fn initialize_user_info(&mut self) -> Result<()> {

        // Initialize counter values
        self.accounts.user_info.account_counter = 0;
        self.accounts.user_info.del_counter = 0;

        // Terms of service, csam tracker
        self.accounts.user_info.agreed_to_tos = true;
        self.accounts.user_info.lifetime_bad_csam = false;

        Ok(())
    }
    fn check_csam(&mut self) -> bool {
        self.accounts.user_info.lifetime_bad_csam
    }
    fn change_global_storage(&mut self, bytes: u64, mode: Mode) -> Result<()> {
        let bytes = bytes as u128;
        match mode {
            Mode::Increment => {
                self.accounts.storage_config.storage_available = self.accounts.storage_config.storage_available.checked_add(bytes).unwrap();
            },
            Mode::Decrement => {
                // The requirement should be that the bytes is less than what is available
                // but we are planning to no longer tracking this on-chain as of Jun 14.
                // As such, the uploader should sign off on this.
                self.accounts.storage_config.storage_available = self.accounts.storage_config.storage_available.checked_sub(bytes).unwrap();
            },
            Mode::Initialize => {
                self.accounts.storage_config.storage_available = bytes;
            }
        }
        Ok(())
    }
    fn get_shades_per_gib(&mut self) -> Option<u64> {
        Some(self.accounts.storage_config.shades_per_gib)
    }
    fn get_user_token_balance(&mut self) -> Option<u64> {
        Some(self.accounts.owner_1_token_account.amount)
    }
    fn stake_shades(&mut self, shades: u64) -> Result<()> {
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_1_token_account.to_account_info(),
                    to: self.accounts.stake_account.to_account_info(),
                    authority: self.accounts.owner_1.to_account_info(),
                },
            ),
            shades,
        )
    }
    fn increment_account_counter(&mut self) -> Result<()> {
        self.accounts.user_info.account_counter = self
            .accounts
            .user_info
            .account_counter
            .checked_add(1)
            .unwrap();
        Ok(())
    }
}

impl InitializeStorageAccount for Context<'_, '_, '_, '_, InitializeStorageAccountV2<'_>> {
    fn set_immutable(&mut self, boolean: bool) -> Result<()> {
        self.accounts.storage_account.immutable = boolean;
        Ok(())
    }
    fn set_identifier(&mut self, identifier: String ) -> Result<()>{
        self.accounts.storage_account.identifier = identifier;
        Ok(())
    }
    fn set_deletion_flag(&mut self, boolean: bool, epoch: u64) -> Result<()> {
        self.accounts.storage_account.to_be_deleted = boolean;
        self.accounts.storage_account.delete_request_epoch = epoch.try_into().unwrap();
        Ok(())
    }
    fn set_account_counter_seed(&mut self) -> Result<()> {
        self.accounts.storage_account.account_counter_seed = self.get_account_counter();
        Ok(())
    }
    fn change_storage(&mut self, bytes: u64, mode: Mode) -> Result<()> {
        let bytes = self.accounts.storage_config.validate_storage(bytes)?;
        match mode {
            Mode::Increment => {
                self.accounts.storage_account.storage = self.accounts.storage_account.storage.checked_add(bytes).unwrap();
            },
            Mode::Decrement => {
                // The requirement should be that the bytes is less than what is available
                // but we are planning to no longer tracking this on-chain as of Jun 14.
                // As such, the uploader should sign off on this.
                self.accounts.storage_account.storage = self.accounts.storage_account.storage.checked_sub(bytes).unwrap();
            },
            Mode::Initialize => {
                self.accounts.storage_account.storage = bytes;
            }
        }
        Ok(())
    }
    fn set_owner(&mut self) -> Result<()> {
        self.accounts.storage_account.owner_1 = self.accounts.owner_1.key();
        Ok(())
    }
    fn set_owner2(&mut self, owner_2: Option<Pubkey>) -> Result<()>{
        if owner_2.is_some() {
            err!(ErrorCodes::OnlyOneOwnerAllowedInV1_5)
        } else {
            Ok(())
        }
    }
    fn record_genesis(&mut self) -> Result<()> {
        let clock = Clock::get().unwrap();
        self.accounts.storage_account.creation_time = clock.unix_timestamp.try_into().unwrap();
        self.accounts.storage_account.creation_epoch = clock.epoch.try_into().unwrap();
        self.accounts.storage_account.last_fee_epoch = clock.epoch.try_into().unwrap();
        Ok(())

    }
    fn get_account_counter(&mut self) -> u32 {
        self.accounts.user_info.account_counter
    }
    fn initialize_user_info(&mut self) -> Result<()> {

        // Initialize counter values
        self.accounts.user_info.account_counter = 0;
        self.accounts.user_info.del_counter = 0;

        // Terms of service, csam tracker
        self.accounts.user_info.agreed_to_tos = true;
        self.accounts.user_info.lifetime_bad_csam = false;

        Ok(())
    }
    fn check_csam(&mut self) -> bool {
        self.accounts.user_info.lifetime_bad_csam
    }
    fn change_global_storage(&mut self, bytes: u64, mode: Mode) -> Result<()> {
        let bytes = bytes as u128;
        match mode {
            Mode::Increment => {
                self.accounts.storage_config.storage_available = self.accounts.storage_config.storage_available.checked_add(bytes).unwrap();
            },
            Mode::Decrement => {
                // The requirement should be that the bytes is less than what is available
                // but we are planning to no longer tracking this on-chain as of Jun 14.
                // As such, the uploader should sign off on this.
                self.accounts.storage_config.storage_available = self.accounts.storage_config.storage_available.checked_sub(bytes).unwrap();
            },
            Mode::Initialize => {
                self.accounts.storage_config.storage_available = bytes;
            }
        }
        Ok(())
    }
    fn get_shades_per_gib(&mut self) -> Option<u64> {
        Some(self.accounts.storage_config.shades_per_gib)
    }
    fn get_user_token_balance(&mut self) -> Option<u64> {
        Some(self.accounts.owner_1_token_account.amount)
    }
    fn stake_shades(&mut self, shades: u64) -> Result<()> {
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_1_token_account.to_account_info(),
                    to: self.accounts.stake_account.to_account_info(),
                    authority: self.accounts.owner_1.to_account_info(),
                },
            ),
            shades,
        )
    }
    fn increment_account_counter(&mut self) -> Result<()> {
        self.accounts.user_info.account_counter = self
            .accounts
            .user_info
            .account_counter
            .checked_add(1)
            .unwrap();
        Ok(())
    }
}



#[test]
fn test_v1_base_size(){

    use std::mem::size_of;
    assert_eq!(
        size_of::<StorageAccount>(),
        184
    );
}

#[test]
fn test_v2_base_size(){

    use std::mem::size_of;
    assert_eq!(
        size_of::<StorageAccountV2>(),
        88
    );
}