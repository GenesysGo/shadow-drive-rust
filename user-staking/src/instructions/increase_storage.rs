use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    initialize_account::{StorageAccount, StorageAccountV2, ShadowDriveStorageAccount}, initialize_config::StorageConfig,
};

/// This is the function that handles the `increase_storage` ix
pub fn handler(
    mut ctx: impl IncreaseStorage,
    additional_storage: u64
) -> Result<()> {

    // // Check if account is immutable
    // require!(
    //     !ctx.check_immutable(),
    //     ErrorCodes::StorageAccountMarkedImmutable
    // );

    // Require nonzero change
    require!(additional_storage > 0, ErrorCodes::NoStorageIncrease);

    msg!(
        "Increasing storage on StorageAccount: {}",
        ctx.get_identifier()
    );
    {
        ctx.add_storage(additional_storage)?
    }

    msg!("Charging user for storage");
    {
        ctx.charge_user(additional_storage)?
    }

    Ok(())
}

#[derive(Accounts)]
/// This `IncreaseStorage` context is used in the instruction which allow users to
/// expand the storage available in a storage account.
pub struct IncreaseStorageV1<'info> {
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
        constraint = !storage_account.immutable,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_ata.mint == shdw::ID)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// This token account serves as the account which holds user's stake for file storage.
    #[account(
        mut,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// Uploader needs to sign off on increase storage
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

#[derive(Accounts)]
/// This `IncreaseStorage` context is used in the instruction which allow users to
/// expand the storage available in a storage account.
pub struct IncreaseStorageV2<'info> {
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
        constraint = !storage_account.immutable,
    )]
    pub storage_account: Box<Account<'info, StorageAccountV2>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_ata.mint == shdw::ID)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// This token account serves as the account which holds user's stake for file storage.
    #[account(
        mut,
        seeds = [
            "stake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// Uploader needs to sign off on increase storage
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


#[derive(Accounts)]
/// This `IncreaseStorage` context is used in the instruction which allow users to
/// expand the storage available in a storage account.
pub struct IncreaseImmutableStorageV1<'info> {
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
        constraint = storage_account.immutable,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

    /// Wallet that receives storage fees
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_ata.mint == shdw::ID)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// Uploader needs to sign off on increase storage
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

#[derive(Accounts)]
/// This `IncreaseStorage` context is used in the instruction which allow users to
/// expand the storage available in a storage account.
pub struct IncreaseImmutableStorageV2<'info> {
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
        constraint = storage_account.immutable,
    )]
    pub storage_account: Box<Account<'info, StorageAccountV2>>,

    /// Wallet that receives storage fees
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// This is the user's token account with which they are staking
    #[account(mut, constraint = owner_ata.mint == shdw::ID)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// Uploader needs to sign off on increase storage
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


fn safe_amount(additional_storage: u64, rate_per_gib: u64) -> Result<u64> {
    let result = (additional_storage as u128)
        .checked_mul(rate_per_gib as u128)
        .unwrap()
        .checked_div(BYTES_PER_GIB as u128)
        .unwrap();
    if (result as u64) as u128 == result {
        Ok(result as u64)
    } else {
        err!(ErrorCodes::UnsignedIntegerCastFailed)
    }
}

pub trait IncreaseStorage {
    fn check_immutable(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn add_storage(&mut self, additional_storage: u64) -> Result<()>;
    fn charge_user(&mut self, additional_storage: u64) -> Result<()>;
}

impl IncreaseStorage for Context<'_,'_,'_,'_, IncreaseStorageV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn add_storage(&mut self, additional_storage: u64) -> Result<()> {
        // Update total storage and storage_available
        msg!(
            "Initial storage: {}",
            self.accounts.storage_account.storage,
        );
        self.accounts.storage_account.storage = self.accounts.storage_account
            .storage
            .checked_add(additional_storage)
            .unwrap();
        msg!(
            "New storage: {}",
            self.accounts.storage_account.storage,
        );

        Ok(())
    }
    fn charge_user(&mut self, additional_storage: u64) -> Result<()> {

        // Compute cost (at least one shade)
        let cost = safe_amount(additional_storage, self.accounts.storage_config.shades_per_gib)?.max(1);

        // Transfer SHDW
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_ata.to_account_info(),
                    to: self.accounts.stake_account.to_account_info(),
                    authority: self.accounts.owner.to_account_info(),
                },
            ),
            cost,
        )
    }
}

impl IncreaseStorage for Context<'_,'_,'_,'_, IncreaseStorageV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn add_storage(&mut self, additional_storage: u64) -> Result<()> {
        // Update total storage and storage_available
        msg!(
            "Initial storage: {}",
            self.accounts.storage_account.storage,
        );
        self.accounts.storage_account.storage = self.accounts.storage_account
            .storage
            .checked_add(additional_storage)
            .unwrap();
        msg!(
            "New storage: {}",
            self.accounts.storage_account.storage,
        );

        Ok(())
    }
    fn charge_user(&mut self, additional_storage: u64) -> Result<()> {

        // Compute cost (at least one shade)
        let cost = safe_amount(additional_storage, self.accounts.storage_config.shades_per_gib)?.max(1);

        // Transfer SHDW
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_ata.to_account_info(),
                    to: self.accounts.stake_account.to_account_info(),
                    authority: self.accounts.owner.to_account_info(),
                },
            ),
            cost,
        )
    }
}


impl IncreaseStorage for Context<'_,'_,'_,'_, IncreaseImmutableStorageV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn add_storage(&mut self, additional_storage: u64) -> Result<()> {
        // Update total storage and storage_available
        msg!(
            "Initial storage: {}",
            self.accounts.storage_account.storage,
        );
        self.accounts.storage_account.storage = self.accounts.storage_account
            .storage
            .checked_add(additional_storage)
            .unwrap();
        msg!(
            "New storage: {}",
            self.accounts.storage_account.storage,
        );

        Ok(())
    }
    fn charge_user(&mut self, additional_storage: u64) -> Result<()> {

        // Compute cost (at least one shade)
        let cost = safe_amount(additional_storage, self.accounts.storage_config.shades_per_gib)?.max(1);

        // Transfer SHDW
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_ata.to_account_info(),
                    to: self.accounts.emissions_wallet.to_account_info(),
                    authority: self.accounts.owner.to_account_info(),
                },
            ),
            cost,
        )
    }
}

impl IncreaseStorage for Context<'_,'_,'_,'_, IncreaseImmutableStorageV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn add_storage(&mut self, additional_storage: u64) -> Result<()> {
        // Update total storage and storage_available
        msg!(
            "Initial storage: {}",
            self.accounts.storage_account.storage,
        );
        self.accounts.storage_account.storage = self.accounts.storage_account
            .storage
            .checked_add(additional_storage)
            .unwrap();
        msg!(
            "New storage: {}",
            self.accounts.storage_account.storage,
        );

        Ok(())
    }
    fn charge_user(&mut self, additional_storage: u64) -> Result<()> {

        // Compute cost (at least one shade)
        let cost = safe_amount(additional_storage, self.accounts.storage_config.shades_per_gib)?.max(1);

        // Transfer SHDW
        anchor_spl::token::transfer(
            CpiContext::new(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.owner_ata.to_account_info(),
                    to: self.accounts.emissions_wallet.to_account_info(),
                    authority: self.accounts.owner.to_account_info(),
                },
            ),
            cost,
        )
    }
}


