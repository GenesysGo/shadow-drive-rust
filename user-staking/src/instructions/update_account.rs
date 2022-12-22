use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction::transfer;
use anchor_spl::token::Mint;

use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    initialize_account::{ShadowDriveStorageAccount, StorageAccount, StorageAccountV2}, initialize_config::StorageConfig,
};

/// This is the function that handles the `update_account` ix
pub fn handler(
    mut ctx: impl UpdateAccount,
    identifier: Option<String>,
    owner_2: Option<Pubkey>,
    // owner_3: Option<Pubkey>,
    // owner_4: Option<Pubkey>,
) -> Result<()> {
    // Check if account is immutable
    require!(
        !ctx.check_immutable(),
        ErrorCodes::StorageAccountMarkedImmutable
    );

    msg!(
        "Updating StorageAccount: {}",
        ctx.get_identifier()
    );

    // Change identifier
    ctx.update_identifier(identifier)?;

    // Update owners
    if let Some(owner_2) = owner_2 {
        ctx.update_owner2(owner_2)?;
    }

    Ok(())
}

#[derive(Accounts)]
/// This `UpdateAccount` context is used in the instruction that allows users to
/// update storage account metadata, such as the identifier and owner 2-4 pubkeys.
pub struct UpdateAccountV1<'info> {
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

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
/// This `UpdateAccount` context is used in the instruction that allows users to
/// update storage account metadata, such as the identifier and owner 2-4 pubkeys.
pub struct UpdateAccountV2<'info> {
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

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Account<'info, Mint>,

    /// System Program
    pub system_program: Program<'info, System>,
}

pub trait UpdateAccount {
    fn check_immutable(&self) -> bool;
    fn get_identifier(&self) -> String;
    fn update_identifier(&mut self, identifier: Option<String>) -> Result<()>;
    fn update_owner2(&mut self, owner_2: Pubkey) -> Result<()>;
}

impl UpdateAccount for Context<'_,'_,'_,'_, UpdateAccountV1<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn update_identifier(&mut self, identifier: Option<String>) -> Result<()> {

        if let Some(identifier) = identifier {

            let account_info = self.accounts.storage_account.to_account_info();

            // Get new pda size
            let new_size = crate::instructions::initialize_account::calc_v1_storage(&identifier);
            let current_size =  crate::instructions::initialize_account::calc_v1_storage(&self.accounts.storage_account.identifier);

            // Update identifier
            msg!("Renaming account from {} to {}", self.accounts.storage_account.identifier, identifier);
            self.accounts.storage_account.identifier = identifier;

            // calculate rent diff
            let rent = Rent::default();
            let min_balance: u64 = rent.minimum_balance(new_size);
            let current_balance: u64 = account_info.lamports();

        
            // Transfer from account to user if rent cost is decreasing
            if current_balance > min_balance {


                let mut user_balance = self.accounts.owner.try_borrow_mut_lamports()?;
                let mut account_balance = account_info.try_borrow_mut_lamports()?;

                let diff = current_balance.checked_sub(min_balance).unwrap();
                **user_balance = user_balance.checked_add(diff).unwrap();
                **account_balance = account_balance.checked_sub(diff).unwrap();

            // Otherwise charge user
            } else if current_balance < min_balance {

                // Construct transfer `Instruction`
                let ix = transfer(
                    &self.accounts.owner.key(),
                    &self.accounts.storage_account.key(),
                    min_balance.checked_sub(current_balance).unwrap(),
                );

                // Invoke
                invoke(
                    &ix,
                    &[
                        self.accounts.owner.to_account_info(),
                        self.accounts.storage_account.to_account_info(),
                    ],
                )?;
            }

            if new_size != current_size {
                account_info.realloc(new_size, false)?;
            }
        }


        Ok(())
    }
    fn update_owner2(&mut self, owner_2: Pubkey) -> Result<()> {
        self.accounts.storage_account.owner_2 = owner_2;
        Ok(())
    }
}

impl UpdateAccount for Context<'_,'_,'_,'_, UpdateAccountV2<'_>> {
    fn check_immutable(&self) -> bool {
        self.accounts.storage_account.check_immutable()
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.get_identifier()
    }
    fn update_identifier(&mut self, identifier: Option<String>) -> Result<()> {

        if let Some(identifier) = identifier {

            let account_info = self.accounts.storage_account.to_account_info();

            // Get new pda size
            let new_size = crate::instructions::initialize_account::calc_v2_storage(&identifier);
            let current_size =  crate::instructions::initialize_account::calc_v2_storage(&self.accounts.storage_account.identifier);

            // Update identifier
            msg!("Renaming account from {} to {}", self.accounts.storage_account.identifier, identifier);
            self.accounts.storage_account.identifier = identifier;

            // calculate rent diff
            let rent = Rent::default();
            let min_balance: u64 = rent.minimum_balance(new_size);
            let current_balance: u64 = account_info.lamports();

        
            // Transfer from account to user if rent cost is decreasing
            if current_balance > min_balance {


                let mut user_balance = self.accounts.owner.try_borrow_mut_lamports()?;
                let mut account_balance = account_info.try_borrow_mut_lamports()?;

                let diff = current_balance.checked_sub(min_balance).unwrap();
                **user_balance = user_balance.checked_add(diff).unwrap();
                **account_balance = account_balance.checked_sub(diff).unwrap();

            // Otherwise charge user
            } else if current_balance < min_balance {

                // Construct transfer `Instruction`
                let ix = transfer(
                    &self.accounts.owner.key(),
                    &self.accounts.storage_account.key(),
                    min_balance.checked_sub(current_balance).unwrap(),
                );

                // Invoke
                invoke(
                    &ix,
                    &[
                        self.accounts.owner.to_account_info(),
                        self.accounts.storage_account.to_account_info(),
                    ],
                )?;
            }

            if new_size != current_size {
                account_info.realloc(new_size, false)?;
            }
        }
        
        Ok(())
    }
    fn update_owner2(&mut self, _owner_2: Pubkey) -> Result<()> {
        err!(ErrorCodes::OnlyOneOwnerAllowedInV1_5)
    }
}

