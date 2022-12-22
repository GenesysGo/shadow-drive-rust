use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2}, initialize_config::StorageConfig,
};

/// This is the function that handles the `refresh_stake` ix
pub fn handler(mut ctx: impl RefreshStake) -> Result<()> {

    // Check if account is immutable
    require!(
        !ctx.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    msg!("Charging user for storage");
    ctx.refresh_stake()?;

    // Unmarks for deletion if marked for deletion"
    ctx.unmark_delete();

    Ok(())
}

#[derive(Accounts)]
/// This `RefreshStake` context is used in the instruction which allow users to
/// top off their stake account
pub struct RefreshStakeV1<'info> {
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
            &storage_account.key().to_bytes()
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
/// This `RefreshStake` context is used in the instruction which allow users to
/// top off their stake account
pub struct RefreshStakeV2<'info> {
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
            &storage_account.key().to_bytes()
        ],
        bump,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,
}

fn safe_amount(storage: u64, rate_per_gib: u128) -> Result<u64> {
    let result = (storage as u128)
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

pub trait RefreshStake {
    fn check_immutable(&self) -> bool;
    fn refresh_stake(&mut self) -> Result<()>;
    fn unmark_delete(&mut self);
}

impl RefreshStake for Context<'_, '_, '_, '_, RefreshStakeV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn refresh_stake(&mut self) -> Result<()> {

        let storage_config = &self.accounts.storage_config;
        let storage_account = &self.accounts.storage_account;
        let stake_account = &self.accounts.stake_account;

        // Compute total cost of storage
        let total_cost: u64 = safe_amount(
            storage_account.storage,
            storage_config.shades_per_gib as u128,
        )?
        .max(1);

        // Compute refresh amount, the difference between total_cost and stake balance
        let refresh_amount: u64 = total_cost.saturating_sub(stake_account.amount);

        // Transfer SHDW
        if refresh_amount > 0 {
            anchor_spl::token::transfer(
                CpiContext::new(
                    self.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.accounts.owner_ata.to_account_info(),
                        to: self.accounts.stake_account.to_account_info(),
                        authority: self.accounts.owner.to_account_info(),
                    },
                ),
                refresh_amount,
            )?;
        }

        Ok(())
    }

    fn unmark_delete(&mut self) {

        let storage_account = &mut self.accounts.storage_account;

        if storage_account.to_be_deleted {
            msg!("Account was marked for deletion. Unmarking");

            // Update deletion flag and reset request time
            storage_account.to_be_deleted = false;
            storage_account.delete_request_epoch = 0;
        }
    }
}


impl RefreshStake for Context<'_, '_, '_, '_, RefreshStakeV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn refresh_stake(&mut self) -> Result<()> {

        let storage_config = &self.accounts.storage_config;
        let storage_account = &self.accounts.storage_account;
        let stake_account = &self.accounts.stake_account;

        // Compute total cost of storage
        let total_cost: u64 = safe_amount(
            storage_account.storage,
            storage_config.shades_per_gib as u128,
        )?
        .max(1);

        // Compute refresh amount, the difference between total_cost and stake balance
        let refresh_amount: u64 = total_cost.saturating_sub(stake_account.amount);

        // Transfer SHDW
        if refresh_amount > 0 {
            anchor_spl::token::transfer(
                CpiContext::new(
                    self.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.accounts.owner_ata.to_account_info(),
                        to: self.accounts.stake_account.to_account_info(),
                        authority: self.accounts.owner.to_account_info(),
                    },
                ),
                refresh_amount,
            )?;
        }

        Ok(())
    }

    fn unmark_delete(&mut self) {

        let storage_account = &mut self.accounts.storage_account;

        if storage_account.to_be_deleted {
            msg!("Account was marked for deletion. Unmarking");

            // Update deletion flag and reset request time
            storage_account.to_be_deleted = false;
            storage_account.delete_request_epoch = 0;
        }
    }
}
