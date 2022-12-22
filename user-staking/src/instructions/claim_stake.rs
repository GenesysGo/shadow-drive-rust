use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    decrease_storage::UnstakeInfo, initialize_account::{StorageAccount, StorageAccountV2},
    initialize_config::StorageConfig,
};

/// This is the function that handles the `claim_stake` instruction
pub fn handler(
    ctx: impl ClaimStake
) -> Result<()> {

    // Must wait to unstake
    let clock = Clock::get().unwrap();
    let current_time = clock.unix_timestamp;
    let current_epoch = clock.epoch;
    let (unstake_time, unstake_epoch) = ctx.get_unstake_time();
    
    require!(
        current_time.checked_sub(unstake_time).unwrap() >= UNSTAKE_TIME_PERIOD,
        ErrorCodes::ClaimingStakeTooSoon
    );
    require!(
        current_epoch.checked_sub(unstake_epoch).unwrap() >= UNSTAKE_EPOCH_PERIOD,
        ErrorCodes::ClaimingStakeTooSoon
    );

    msg!("Transferring all outstanding unstake funds to user");
    {
        ctx.transfer_shades_to_user()?;
    }

    Ok(())
}

type Time = i64;
type Epoch = u64;
pub trait ClaimStake {
    fn get_unstake_time(&self) -> (Time, Epoch);
    fn transfer_shades_to_user(&self) -> Result<()>;
}


#[derive(Accounts)]
/// This `ClaimStake` context is used in the instruction which allow users to
/// claim funds after waiting an appropriate amount of time.
pub struct ClaimStakeV2<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account. Only used here for the key
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

    /// Account which stores time, epoch last unstaked. Close upon successful unstake.
    #[account(
        mut,
        close = owner,
        seeds = [
            "unstake-info".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_info: Box<Account<'info, UnstakeInfo>>,

    /// Account which stores SHDW when unstaking.  Close upon successful unstake.
    #[account(
        mut,
        seeds = [
            "unstake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_account: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, address=unstake_info.unstaker)]
    pub owner: Signer<'info>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(
        mut,
        constraint = {
            owner_ata.owner == owner.key()
            && owner_ata.mint == token_mint.key()
        }
    )]
    pub owner_ata: Account<'info, TokenAccount>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Programn
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
/// This `ClaimStake` context is used in the instruction which allow users to
/// claim funds after waiting an appropriate amount of time.
pub struct ClaimStakeV1<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account. Only used here for the key
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

    /// Account which stores time, epoch last unstaked. Close upon successful unstake.
    #[account(
        mut,
        close = owner,
        seeds = [
            "unstake-info".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_info: Box<Account<'info, UnstakeInfo>>,

    /// Account which stores SHDW when unstaking.  Close upon successful unstake.
    #[account(
        mut,
        seeds = [
            "unstake-account".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub unstake_account: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, address=unstake_info.unstaker)]
    pub owner: Signer<'info>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(
        mut,
        constraint = {
            owner_ata.owner == owner.key()
            && owner_ata.mint == token_mint.key()
        }
    )]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Programn
    pub token_program: Program<'info, Token>,
}


impl ClaimStake for Context<'_, '_, '_, '_, ClaimStakeV1<'_>> {
    fn get_unstake_time(&self) -> (Time, Epoch) {
        (self.accounts.unstake_info.time_last_unstaked, self.accounts.unstake_info.epoch_last_unstaked)
    }
    fn transfer_shades_to_user(&self) -> Result<()> {

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Transfer shades to user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.unstake_account.to_account_info(),
                    to: self.accounts.owner_ata.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            self.accounts.unstake_account.amount,
        )?;

        // Close unstake account
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.unstake_account.to_account_info(),
                destination: self.accounts.owner_ata.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))
    }
}

impl ClaimStake for Context<'_, '_, '_, '_, ClaimStakeV2<'_>> {
    fn get_unstake_time(&self) -> (Time, Epoch) {
        (self.accounts.unstake_info.time_last_unstaked, self.accounts.unstake_info.epoch_last_unstaked)
    }
    fn transfer_shades_to_user(&self) -> Result<()> {

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Transfer shades to user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.unstake_account.to_account_info(),
                    to: self.accounts.owner_ata.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            self.accounts.unstake_account.amount,
        )?;

        // Close unstake account
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.unstake_account.to_account_info(),
                destination: self.accounts.owner_ata.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))
    }

}