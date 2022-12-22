use crate::constants::shdw;
use crate::instructions::{
    initialize_account::{StorageAccount, StorageAccountV2, UserInfo},
    initialize_config::StorageConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

/// This is the function that handles the `bad_csam` instruction
pub fn handler(mut ctx: impl BadCsam, storage_available: u64) -> Result<()> {

    msg!("Transfer all funds in relevant stake account to emissions wallet");
    ctx.transfer_stake_to_emissions_wallet()?;

    msg!("Update Global Storage");
    ctx.update_global_storage(storage_available)?;

    msg!("Update User Info");
    ctx.flag_account_bad_csam();

    Ok(())
}


#[derive(Accounts)]
/// This `BadCsam` context is used in the instruction that handles (closes)
/// the accounts of a user that attmepted to upload a file that did not pass a csam scan.
pub struct BadCsam1<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
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
        mut,
        seeds = [
            "user-info".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
        ],
        bump,
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// Parent storage account.
    #[account(
        mut,
        close = uploader,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes(),
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccount>>,

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

    /// Admin/uploader
    #[account(mut, constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

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
/// This `BadCsam` context is used in the instruction that handles (closes)
/// the accounts of a user that attmepted to upload a file that did not pass a csam scan.
pub struct BadCsam2<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
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
        mut,
        seeds = [
            "user-info".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
        ],
        bump,
    )]
    pub user_info: Box<Account<'info, UserInfo>>,

    /// Parent storage account.
    #[account(
        mut,
        close = uploader,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes(),
        ],
        bump,
    )]
    pub storage_account: Box<Account<'info, StorageAccountV2>>,

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

    /// Admin/uploader
    #[account(mut, constraint = uploader.key() == storage_config.uploader)]
    pub uploader: Signer<'info>,

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


pub trait BadCsam {
    fn transfer_stake_to_emissions_wallet(&mut self) -> Result<()>;
    fn update_global_storage(&mut self, storage_available: u64) -> Result<()>;
    fn flag_account_bad_csam(&mut self);
}

impl BadCsam for Context<'_, '_, '_, '_, BadCsam1<'_>> {
    fn transfer_stake_to_emissions_wallet(&mut self) -> Result<()>{
        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Transfer all funds from user's stake account to emissions wallet and close account
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.emissions_wallet.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            self.accounts.stake_account.amount,
        )?;
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.stake_account.to_account_info(),
                destination: self.accounts.emissions_wallet.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))?;

        Ok(())
    }
    fn update_global_storage(&mut self, storage_available: u64) -> Result<()>{

        let storage_config = &mut self.accounts.storage_config;
        let storage_account = &mut self.accounts.storage_account;

        // Trim storage on storage_account
        storage_account.storage = storage_account
            .storage
            .checked_sub(storage_available)
            .unwrap();

        // Add trimmed storage back to global Shadow Drive storage
        storage_config.storage_available = storage_config
            .storage_available
            .checked_add(storage_available.into())
            .unwrap();

        Ok(())
    }
    fn flag_account_bad_csam(&mut self) {
        self.accounts.user_info.lifetime_bad_csam = true;
    }
}

impl BadCsam for Context<'_, '_, '_, '_, BadCsam2<'_>> {
    fn transfer_stake_to_emissions_wallet(&mut self) -> Result<()>{
        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Transfer all funds from user's stake account to emissions wallet and close account
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.emissions_wallet.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            self.accounts.stake_account.amount,
        )?;
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.stake_account.to_account_info(),
                destination: self.accounts.emissions_wallet.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))?;

        Ok(())
    }
    fn update_global_storage(&mut self, storage_available: u64) -> Result<()>{

        let storage_config = &mut self.accounts.storage_config;
        let storage_account = &mut self.accounts.storage_account;

        // Trim storage on storage_account
        storage_account.storage = storage_account
            .storage
            .checked_sub(storage_available)
            .unwrap();

        // Add trimmed storage back to global Shadow Drive storage
        storage_config.storage_available = storage_config
            .storage_available
            .checked_add(storage_available.into())
            .unwrap();

        Ok(())
    }
    fn flag_account_bad_csam(&mut self) {
        self.accounts.user_info.lifetime_bad_csam = true;
    }
}

