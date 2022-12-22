use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    crank::crank,
    initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2, UserInfo},
    initialize_config::StorageConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::convert::TryFrom;

/// This is the function that handles the `delete_account` ix
pub fn handler(mut ctx: impl DeleteAccount) -> Result<()> {
    // Cannot request to delete storage account if immutable
    require!(
        !ctx.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    // Cannot delete a storage account which still has associated file accounts
    // require_eq!(
    //     ctx.accounts.storage_account.init_counter,
    //     ctx.accounts.storage_account.del_counter,
    //     ErrorCodes::NonzeroRemainingFileAccounts
    // );

    // Cannot delete a storage account not marked for deletion
    require!(
        ctx.check_delete_flag(),
        ErrorCodes::AccountNotMarkedToBeDeleted
    );

    // Cannot delete a storage account in grace period
    require!(
        ctx.check_grace_period(),
        ErrorCodes::AccountStillInGracePeriod
    );

    msg!(
        "Deleting StorageAccount account: {}",
        ctx.get_identifier(),
    );
    // account is marked with `close` in Context<_>

    msg!("Returning stake to user");
    ctx.return_stake()?;

    msg!("Updating global storage on StorageConfig account");
    ctx.update_global_storage()?;

    msg!("Update User Info");
    ctx.increment_del_counter()?;

    Ok(())
}

#[derive(Accounts)]
/// This `DeleteAccount` context is used in the instruction that allows
/// admins to delete user storage accounts.
pub struct DeleteAccountV1<'info> {
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
        close = owner,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes()
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

    /// File owner, user
    /// CHECK: There is a constraint that checks whether this account is an owner.
    /// Also, our uploader keys are signing this transaction so presuamably we would only provide a good key.
    /// We also may not need this account at all.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: AccountInfo<'info>,

    /// This is the user's token account, presumably with which they staked
    #[account(
        mut,
        constraint = {
            storage_account.is_owner(shdw_payer.owner)
            && shdw_payer.mint == token_mint.key()
        },
    )]
    pub shdw_payer: Account<'info, TokenAccount>,

    /// Admin/uploader
    #[account(constraint = uploader.key() == storage_config.uploader)]
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
/// This `DeleteAccount` context is used in the instruction that allows
/// admins to delete user storage accounts.
pub struct DeleteAccountV2<'info> {
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
        close = owner,
        seeds = [
            "storage-account".as_bytes(),
            &storage_account.owner_1.key().to_bytes(),
            &storage_account.account_counter_seed.to_le_bytes()
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

    /// File owner, user
    /// CHECK: There is a constraint that checks whether this account is an owner.
    /// Also, our uploader keys are signing this transaction so presuamably we would only provide a good key.
    /// We also may not need this account at all.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: AccountInfo<'info>,

    /// This is the user's token account, presumably with which they staked
    #[account(
        mut,
        constraint = {
            storage_account.is_owner(shdw_payer.owner)
            && shdw_payer.mint == token_mint.key()
        },
    )]
    pub shdw_payer: Account<'info, TokenAccount>,

    /// Admin/uploader
    #[account(constraint = uploader.key() == storage_config.uploader)]
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


pub trait DeleteAccount {
    fn check_immutable(&self) -> bool;
    fn check_delete_flag(&self) -> bool;
    fn check_grace_period(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn return_stake(&mut self) -> Result<()>;
    fn update_global_storage(&mut self) -> Result<()>;
    fn increment_del_counter(&mut self) -> Result<()>;
}

impl DeleteAccount for Context<'_, '_, '_, '_, DeleteAccountV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool{
        self.accounts.storage_account.check_delete_flag()
    }
    fn check_grace_period(&self) -> bool {
        u32::try_from(Clock::get().unwrap().epoch).unwrap() >= self.accounts
            .storage_account
            .delete_request_epoch
            .checked_add(DELETION_GRACE_PERIOD as u32)
            .unwrap()
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn return_stake(&mut self) -> Result<()> {

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Initalize return amount
        let mut return_amount = self.accounts.stake_account.amount;

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
            &self.accounts.emissions_wallet,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            // Subtract fee from return
            let fee = emission_fee.checked_add(crank_fee).unwrap();
            return_amount = return_amount.saturating_sub(fee);
        }

        // Return funds to user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.shdw_payer.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            return_amount,
        )?;

        Ok(())
    }
    fn update_global_storage(&mut self) -> Result<()> {
        
        let storage_config = &mut self.accounts.storage_config;

        // Increase storage available
        storage_config.storage_available = storage_config
            .storage_available
            .checked_add(self.accounts.storage_account.storage as u128)
            .unwrap();

        Ok(())
    }
    fn increment_del_counter(&mut self) -> Result<()> {
        
        // Increment delete counter
        self.accounts.user_info.del_counter = self.accounts.user_info.del_counter.checked_add(1).unwrap();

        Ok(())
    }
}


impl DeleteAccount for Context<'_, '_, '_, '_, DeleteAccountV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn check_delete_flag(&self) -> bool{
        self.accounts.storage_account.check_delete_flag()
    }
    fn check_grace_period(&self) -> bool {
        u32::try_from(Clock::get().unwrap().epoch).unwrap() >= self.accounts
            .storage_account
            .delete_request_epoch
            .checked_add(DELETION_GRACE_PERIOD as u32)
            .unwrap()
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn return_stake(&mut self) -> Result<()> {

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Initalize return amount
        let mut return_amount = self.accounts.stake_account.amount;

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
            &self.accounts.emissions_wallet,
            *self.bumps.get("storage_config").unwrap(),
        )? {
            // Subtract fee from return
            let fee = emission_fee.checked_add(crank_fee).unwrap();
            return_amount = return_amount.saturating_sub(fee);
        }

        // Return funds to user
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                self.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: self.accounts.stake_account.to_account_info(),
                    to: self.accounts.shdw_payer.to_account_info(),
                    authority: self.accounts.storage_config.to_account_info(),
                },
                signer_seeds,
            ),
            return_amount,
        )?;

        Ok(())
    }
    fn update_global_storage(&mut self) -> Result<()> {
        
        let storage_config = &mut self.accounts.storage_config;

        // Increase storage available
        storage_config.storage_available = storage_config
            .storage_available
            .checked_add(self.accounts.storage_account.storage as u128)
            .unwrap();

        Ok(())
    }
    fn increment_del_counter(&mut self) -> Result<()> {
        
        // Increment delete counter
        self.accounts.user_info.del_counter = self.accounts.user_info.del_counter.checked_add(1).unwrap();

        Ok(())
    }
}
