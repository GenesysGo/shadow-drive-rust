use crate::constants::{shdw, BYTES_PER_GIB};
use crate::errors::ErrorCodes;
use crate::instructions::{initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2}, initialize_config::StorageConfig};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::convert::TryInto;

/// This is the function that handles the `crank` instruction
pub fn handler(ctx: impl Crank) -> Result<()> {
    ctx.crank()
}

#[derive(Accounts)]
/// This `Crank` context is used in the instruction that allows
/// admins to delete user storage accounts.
pub struct CrankV1<'info> {
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
    pub storage_account: Account<'info, StorageAccount>,

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

    /// Cranker
    #[account(mut)]
    pub cranker: Signer<'info>,

    /// Cranker's ATA
    #[account(
        mut,
        constraint = {
            cranker_ata.owner == cranker.key()
            && cranker_ata.mint == token_mint.key()
        }
    )]
    pub cranker_ata: Box<Account<'info, TokenAccount>>,

    /// This token accountis the SHDW operator emissions wallet
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
/// This `Crank` context is used in the instruction that allows
/// admins to delete user storage accounts.
pub struct CrankV2<'info> {
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

    /// Cranker
    #[account(mut)]
    pub cranker: Signer<'info>,

    /// Cranker's ATA
    #[account(
        mut,
        constraint = {
            cranker_ata.owner == cranker.key()
            && cranker_ata.mint == token_mint.key()
        }
    )]
    pub cranker_ata: Box<Account<'info, TokenAccount>>,

    /// This token accountis the SHDW operator emissions wallet
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,
}

/// This public fn is a crank fn that can be called within this program
/// to collect fees from some specified user's stake account.
pub fn crank<'info, T: ShadowDriveStorageAccount + AccountSerialize + AccountDeserialize + Owner + Clone>(
    storage_config: &Account<'info, StorageConfig>,
    storage_account: &mut Account<'info, T>,
    storage_account_info: AccountInfo<'info>,
    emissions_wallet: &Account<'info, TokenAccount>,
    stake_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    cranker_ata: &Account<'info, TokenAccount>,
    storage_config_bump: u8,
) -> Result<Option<(u64, u64)>> {
    // Mutability checks. Added for checks when called from other instructions.
    require!(
        // Check for mutability on Shadow Drive
        !storage_account.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );
    require!(
        // Check for solana account mutability
        storage_account_info.is_writable,
        ErrorCodes::SolanaStorageAccountNotMutable
    );
    // emissions_wallet, stake_account, cranker_ata checked by anchor_spl::token::transfer

    let clock = Clock::get()?;
    // If mutable account fees are on, charge any outstanding fees.
    if let Some(start_epoch) = storage_config.mutable_fee_start_epoch {
        // When to start charging
        let fee_begin_epoch: u32 = start_epoch.max(storage_account.get_last_fee_epoch());

        // Current epoch
        let fee_end_epoch: u32 = clock.epoch.try_into().unwrap();

        // Calculate fee: (time elapsed) * (shades per mb per epoch) * (bytes / bytes per mb)
        let mut fee = (fee_end_epoch.checked_sub(fee_begin_epoch).unwrap() as u128)
            .checked_mul(storage_config.shades_per_gib_per_epoch as u128)
            .unwrap()
            .checked_mul(storage_account.get_storage() as u128)
            .unwrap()
            .checked_div(BYTES_PER_GIB as u128)
            .unwrap()
            .try_into()
            .unwrap();

        // Fee is limited by stake account balance
        if fee > stake_account.amount {
            // Cap the fee at the stake_account balance
            fee = stake_account.amount;

            // Mark storage account for deletion,
            // since stake account will be empty.
            storage_account.mark_to_delete();
        } else if fee == 0 {
            // If fee_end_epoch == fee_begin_epoch or if rate = 0, stop here
            return Ok(Some((0, 0)));
        }
        msg!(
            "Collecting {} epochs of fees for {} bytes",
            fee_end_epoch.checked_sub(fee_begin_epoch).unwrap(),
            storage_account.get_storage()
        );

        // Update storage_account
        storage_account.update_last_fee_epoch();

        // Split fee into cranker_fee, emissions_fee
        let cranker_fee: u64 = (fee as u128)
            .checked_mul(storage_config.crank_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap()
            .try_into()
            .unwrap();
        let emissions_fee: u64 = fee.checked_sub(cranker_fee).unwrap();
        msg!("Cranker fee: {} shades", cranker_fee);
        msg!("Emissions fee: {} shades", emissions_fee);
        msg!(
            "Storage rate: {} shades per gb per epoch",
            storage_config.shades_per_gib_per_epoch
        );
        // Transfer fees to emissions wallet, cranker
        // Pack seeds
        let storage_config_seeds = ["storage-config".as_bytes(), &[storage_config_bump]];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: stake_account.to_account_info(),
                    to: emissions_wallet.to_account_info(),
                    authority: storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            emissions_fee,
        )?;
        if cranker_fee > 0 {
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: stake_account.to_account_info(),
                        to: cranker_ata.to_account_info(),
                        authority: storage_config.to_account_info(),
                    },
                    signer_seeds,
                ),
                cranker_fee,
            )?;
        }

        Ok(Some((emissions_fee, cranker_fee)))
    } else {
        // Otherwise, crank should do nothing
        Ok(None)
    }
}


pub trait Crank {
    fn crank(self) -> Result<()>;
}

impl Crank for Context<'_,'_,'_,'_,CrankV1<'_>> {
    fn crank(self) -> Result<()> {
        let account_info = self.accounts.storage_account.to_account_info();
        match crank(
            &self.accounts.storage_config,
            &mut self.accounts.storage_account,
            account_info,
            &self.accounts.emissions_wallet,
            &self.accounts.stake_account,
            &self.accounts.token_program,
            &self.accounts.cranker_ata,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            Some(_) => {
                msg!("Crank Turned");
                Ok(())
            }
            None => {
                msg!("Mutable fees are inactive");
                Ok(())
            }
        }
    }
}

impl Crank for Context<'_,'_,'_,'_,CrankV2<'_>> {
    fn crank(self) -> Result<()> {
        let account_info = self.accounts.storage_account.to_account_info();
        match crank(
            &self.accounts.storage_config,
            &mut self.accounts.storage_account,
            account_info,
            &self.accounts.emissions_wallet,
            &self.accounts.stake_account,
            &self.accounts.token_program,
            &self.accounts.cranker_ata,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            Some(_) => {
                msg!("Crank Turned");
                Ok(())
            }
            None => {
                msg!("Mutable fees are inactive");
                Ok(())
            }
        }
    }
}