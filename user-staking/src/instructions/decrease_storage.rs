use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    crank::crank, initialize_account::{StorageAccount, StorageAccountV2, ShadowDriveStorageAccount}, initialize_config::StorageConfig,
};

/// This is the function that handles the `decrease_storage` ix
pub fn handler(
    mut ctx: impl DecreaseStorage,
    remove_storage: u64
) -> Result<()> {
    // Check if account is immutable
    require!(
        !ctx.is_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    msg!("Unstaking funds");
    {
        ctx.unstake(remove_storage)?;
    }

    msg!(
        "Decreasing storage on StorageAccount: {}",
        ctx.get_identifier()
    );
    {
        // Update total storage and storage_available
        // NOTE: Now that we are not tracking storage on-chain in v1.5, this is the wrong condition,
        // as it should check storage_available > remove_storage. It is up to the uploader server to 
        // check this condition! For now, we do this minimal sanity check whether the storage removed
        // is smaller than the total storage on-chain.
        require_gte!(
            ctx.get_current_storage(),
            remove_storage, 
            ErrorCodes::RemovingTooMuchStorage
        );
        ctx.remove_storage(remove_storage)?;
    }

    msg!("Setting unstake info");
    {
        ctx.record_unstake_info()?;
    }

    Ok(())
}

#[derive(Accounts)]
/// This `DecreaseStorage` context is used in the instruction which allow users to begin
/// to unstake funds, decreasing their available storage.
pub struct DecreaseStorageV1<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        mut,
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

    /// Account which stores time, epoch last unstaked
    #[account(
        init_if_needed,
        space = std::mem::size_of::<UnstakeInfo>() + 8,
        payer = owner,
        seeds = [
            "unstake-info".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_info: Box<Account<'info, UnstakeInfo>>,

    /// Account which stores SHDW when unstaking
    #[account(
        init_if_needed,
        payer = owner,
        seeds = [
            "unstake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = storage_config,
    )]
    pub unstake_account: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// User's ATA
    #[account(
        mut,
        constraint = {
            owner_ata.owner == owner.key()
            && owner_ata.mint == token_mint.key()
        }
    )]
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

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// Uploader needs to sign off on decrease storage
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token account holding operator emission funds
    #[account(mut, address = shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
/// This `DecreaseStorage` context is used in the instruction which allow users to begin
/// to unstake funds, decreasing their available storage.
pub struct DecreaseStorageV2<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        mut,
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
    pub storage_account: Account<'info, StorageAccountV2>,

    /// Account which stores time, epoch last unstaked
    #[account(
        init_if_needed,
        space = std::mem::size_of::<UnstakeInfo>() + 8,
        payer = owner,
        seeds = [
            "unstake-info".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_info: Box<Account<'info, UnstakeInfo>>,

    /// Account which stores SHDW when unstaking
    #[account(
        init_if_needed,
        payer = owner,
        seeds = [
            "unstake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = storage_config,
    )]
    pub unstake_account: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// User's ATA
    #[account(
        mut,
        constraint = {
            owner_ata.owner == owner.key()
            && owner_ata.mint == token_mint.key()
        }
    )]
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

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// Uploader needs to sign off on decrease storage
    #[account(constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token account holding operator emission funds
    #[account(mut, address = shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct UnstakeInfo {
    pub time_last_unstaked: i64,
    pub epoch_last_unstaked: u64,
    pub unstaker: Pubkey,
}

fn safe_mul_div(a: u64, b: u64, c: u64) -> Result<u64> {
    let result = (a as u128)
        .checked_mul(b as u128)
        .unwrap()
        .checked_div(c as u128)
        .unwrap();
    if (result as u64) as u128 == result {
        Ok(result as u64)
    } else {
        err!(ErrorCodes::UnsignedIntegerCastFailed)
    }
}


pub trait DecreaseStorage {
    fn is_immutable(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn get_current_storage(&self) -> u64;
    fn unstake(&mut self, remove_storage: u64) -> Result<()>;
    fn remove_storage(&mut self, remove_storage: u64) -> Result<()>;
    fn record_unstake_info(&mut self) -> Result<()>;
}


impl DecreaseStorage for Context<'_,'_,'_,'_, DecreaseStorageV1<'_>> {
    fn is_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn get_current_storage(&self) -> u64 {
        self.accounts.storage_account.storage
    }
    fn unstake(&mut self, remove_storage: u64) -> Result<()> {

        // Compute unstake amount, up to total stake (in case of any rounding issues)
        let mut unstake_amount = safe_mul_div(
            remove_storage,
            self.accounts.stake_account.amount,
            self.accounts.storage_account.storage,
        )
        .unwrap()
        .min(self.accounts.stake_account.amount);

        // Transfer SHDW
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];

        // If mutable account fees are on, charge any outstanding fees.
        // Note: Cranker fee goes to emissions wallet.
        let account_info = self.accounts.storage_account.to_account_info();
        if let Some((emission_fee, crank_fee)) = crank(
            &self.accounts.storage_config,
            &mut self.accounts.storage_account,
            account_info,
            &self.accounts.emissions_wallet,
            &self.accounts.stake_account,
            &self.accounts.token_program,
            &self.accounts.owner_ata,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            // Subtract fee from return
            let fee = emission_fee.checked_add(crank_fee).unwrap();
            unstake_amount = unstake_amount.saturating_sub(fee);
        }

        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.unstake_account.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            unstake_amount,
        )
    }
    fn remove_storage(&mut self, remove_storage: u64) -> Result<()>{
        self.accounts.storage_account.storage = self
            .accounts
            .storage_account
            .storage
            .checked_sub(remove_storage)
            .unwrap();
        Ok(())
    }
    fn record_unstake_info(&mut self) -> Result<()> {

        // Get clock
        let clock = Clock::get().unwrap();

        // Recored time, epoch, unstaker
        self.accounts.unstake_info.time_last_unstaked = clock.unix_timestamp;
        self.accounts.unstake_info.epoch_last_unstaked = clock.epoch;
        self.accounts.unstake_info.unstaker = self.accounts.owner.key();

        Ok(())
    }
}

impl DecreaseStorage for Context<'_,'_,'_,'_, DecreaseStorageV2<'_>> {
    fn is_immutable(&self) -> bool {
        self.accounts.storage_account.immutable
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn get_current_storage(&self) -> u64 {
        self.accounts.storage_account.storage
    }
    fn unstake(&mut self, remove_storage: u64) -> Result<()> {

        // Compute unstake amount, up to total stake (in case of any rounding issues)
        let mut unstake_amount = safe_mul_div(
            remove_storage,
            self.accounts.stake_account.amount,
            self.accounts.storage_account.storage,
        )
        .unwrap()
        .min(self.accounts.stake_account.amount);

        // Transfer SHDW
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];

        // If mutable account fees are on, charge any outstanding fees.
        // Note: Cranker fee goes to emissions wallet.
        let account_info = self.accounts.storage_account.to_account_info();
        if let Some((emission_fee, crank_fee)) = crank(
            &self.accounts.storage_config,
            &mut self.accounts.storage_account,
            account_info,
            &self.accounts.emissions_wallet,
            &self.accounts.stake_account,
            &self.accounts.token_program,
            &self.accounts.owner_ata,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            // Subtract fee from return
            let fee = emission_fee.checked_add(crank_fee).unwrap();
            unstake_amount = unstake_amount.saturating_sub(fee);
        }

        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.unstake_account.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            unstake_amount,
        )
    }
    fn remove_storage(&mut self, remove_storage: u64) -> Result<()>{
        self.accounts.storage_account.storage = self
            .accounts
            .storage_account
            .storage
            .checked_sub(remove_storage)
            .unwrap();
        Ok(())
    }
    fn record_unstake_info(&mut self) -> Result<()> {

        // Get clock
        let clock = Clock::get().unwrap();

        // Recored time, epoch, unstaker
        self.accounts.unstake_info.time_last_unstaked = clock.unix_timestamp;
        self.accounts.unstake_info.epoch_last_unstaked = clock.epoch;
        self.accounts.unstake_info.unstaker = self.accounts.owner.key();

        Ok(())
    }
}