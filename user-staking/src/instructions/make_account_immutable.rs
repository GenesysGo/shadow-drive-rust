use crate::constants::*;
use crate::errors::ErrorCodes;
use crate::instructions::{
    crank::crank, initialize_account::{StorageAccount, StorageAccountV2, ShadowDriveStorageAccount}, initialize_config::StorageConfig,
};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use std::convert::TryInto;

/// This is the function that handles the `make_account_immutable` ix
pub fn handler(
    mut ctx: impl MakeAccountImmutable,
) -> Result<()> {

    // Crank first, if needed and if mutable fees are on
    let new_balance: Shades = ctx.crank()?;

    msg!("Charging user for storage; SPL transfers to emissions wallet");
    {
        ctx.charge_or_return_shades(new_balance)?;
    }

    msg!(
        "Marking Storage Account as immutable: {}",
        ctx.get_identifier()
    );
    {
       ctx.mark_immutable();
    }

    Ok(())
}

#[derive(Accounts)]
/// This `MakeAccountImmutable` context is used in the instruction which allow users to
/// mark a storage account as immutable.
pub struct MakeAccountImmutableV1<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    /// Requires mutability to update global storage counter.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account.
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

    /// This token account is the SHDW operator emissions wallet
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// Uploader needs to sign off on make immutable
    #[account(address=storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// User's token account
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = token_mint,
        associated_token::authority = owner,
    )]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Box<Account<'info, Mint>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Associated Token Program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
/// This `MakeAccountImmutable` context is used in the instruction which allow users to
/// mark a storage account as immutable.
pub struct MakeAccountImmutableV2<'info> {
    /// This is the `StorageConfig` accounts that holds all of the admin, uploader keys.
    /// Requires mutability to update global storage counter.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// Parent storage account.
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

    /// This token account is the SHDW operator emissions wallet
    #[account(mut, address=shdw::emissions_wallet::ID)]
    pub emissions_wallet: Box<Account<'info, TokenAccount>>,

    /// File owner, user, fee-payer
    /// Requires mutability since owner/user is fee payer.
    #[account(mut, constraint=storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    /// User's token account
    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = token_mint,
        associated_token::authority = owner,
    )]
    pub owner_ata: Box<Account<'info, TokenAccount>>,
    
    /// Uploader needs to sign off on make immutable
    #[account(address=storage_config.uploader)]
    pub uploader: Signer<'info>,

    /// Token mint account
    #[account(address = shdw::ID)]
    pub token_mint: Box<Account<'info, Mint>>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Token Program
    pub token_program: Program<'info, Token>,

    /// Associated Token Program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Rent
    pub rent: Sysvar<'info, Rent>,
}


type Shades = u64;

pub trait MakeAccountImmutable {
    fn crank(&mut self) -> Result<Shades>;
    fn charge_or_return_shades(&mut self, new_balance: Shades) -> Result<()>;
    fn get_identifier(&self) -> String;
    fn mark_immutable(&mut self);
}

impl MakeAccountImmutable for Context<'_,'_,'_,'_,MakeAccountImmutableV1<'_>> {
    fn crank(&mut self) -> Result<Shades> {
        let pre_crank_balance = self.accounts.stake_account.amount;
        let new_balance;
        let account_info = self.accounts.storage_account.to_account_info();
        {
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
                msg!("First, crank for any outstanding mutable fees (cranker = user)");
                new_balance = pre_crank_balance
                    .checked_sub(emission_fee.checked_add(crank_fee).unwrap())
                    .unwrap();
            } else {
                new_balance = pre_crank_balance;
            }
        }

        Ok(new_balance)
    }
    fn charge_or_return_shades(&mut self, new_balance: Shades) -> Result<()> {
        let storage_config = &self.accounts.storage_config;
        let storage_account = &self.accounts.storage_account;

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Cost of storage
        let immutable_storage_requested: u128 = storage_account
            .storage
            as u128;
        let cost_of_storage: u64 = immutable_storage_requested
            .checked_mul(storage_config.shades_per_gib as u128)
            .unwrap()
            .checked_div(BYTES_PER_GIB as u128)
            .unwrap()
            .try_into()
            .unwrap();
        msg!(
            "User has {} bytes of storage, costing {} shades",
            immutable_storage_requested,
            cost_of_storage
        );

        if cost_of_storage <= new_balance {
            msg!("User's stake account contains enough funds to cover immutable fee");

            // Compute amount to return to user + amount to transfer to emissions wallet
            let return_amount = new_balance.checked_sub(cost_of_storage).unwrap();
            msg!(
                "Returning {} shades to user, {} to emissions wallet",
                return_amount,
                cost_of_storage,
            );

            // Sanity check
            require_eq!(
                new_balance,
                cost_of_storage.checked_add(return_amount).unwrap(),
                ErrorCodes::InvalidTokenTransferAmounts
            );

            // Transfer to user, if necessary
            if return_amount > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new_with_signer(
                        self.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: self.accounts.stake_account.to_account_info(),
                            to: self.accounts.owner_ata.to_account_info(),
                            authority: self.accounts.storage_config.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    return_amount,
                )
                .map_err(|_| ErrorCodes::FailedToReturnUserFunds)?;
            }

            // Transfer to emissions wallet
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
                cost_of_storage,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWallet)?;
        } else {
            let additional_shades: u64 = cost_of_storage.checked_sub(new_balance).unwrap();
            msg!(
                "User's stake account does not contain enough funds, charging user additional {} shades",
                additional_shades
            );

            // Sanity check
            require_eq!(
                cost_of_storage,
                additional_shades.checked_add(new_balance).unwrap(),
                ErrorCodes::InvalidTokenTransferAmounts
            );

            // Transfer from user
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.accounts.owner_ata.to_account_info(),
                        to: self.accounts.emissions_wallet.to_account_info(),
                        authority: self.accounts.owner.to_account_info(),
                    },
                    signer_seeds,
                ),
                additional_shades,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWalletFromUser)?;

            // Transfer to emissions wallet
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
                new_balance,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWallet)?;
        }

        // Close account after having emptied it
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.stake_account.to_account_info(),
                destination: self.accounts.emissions_wallet.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))
        .map_err(|_| ErrorCodes::FailedToCloseAccount)?;
        
        Ok(())
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn mark_immutable(&mut self) {
        self.accounts.storage_account.immutable = true;
    }
}


impl MakeAccountImmutable for Context<'_,'_,'_,'_,MakeAccountImmutableV2<'_>> {
    fn crank(&mut self) -> Result<Shades> {
        let pre_crank_balance = self.accounts.stake_account.amount;
        let new_balance;
        let account_info = self.accounts.storage_account.to_account_info();
        {
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
                msg!("First, crank for any outstanding mutable fees (cranker = user)");
                new_balance = pre_crank_balance
                    .checked_sub(emission_fee.checked_add(crank_fee).unwrap())
                    .unwrap();
            } else {
                new_balance = pre_crank_balance;
            }
        }

        Ok(new_balance)
    }
    fn charge_or_return_shades(&mut self, new_balance: Shades) -> Result<()> {
        let storage_config = &self.accounts.storage_config;
        let storage_account = &self.accounts.storage_account;

        // Pack seeds
        let storage_config_seeds = [
            "storage-config".as_bytes(),
            &[*self.bumps.get("storage_config").unwrap()],
        ];
        let signer_seeds: &[&[&[u8]]] = &[&storage_config_seeds];

        // Cost of storage
        let immutable_storage_requested: u128 = storage_account
            .storage
            as u128;
        let cost_of_storage: u64 = immutable_storage_requested
            .checked_mul(storage_config.shades_per_gib as u128)
            .unwrap()
            .checked_div(BYTES_PER_GIB as u128)
            .unwrap()
            .try_into()
            .unwrap();
        msg!(
            "User has {} bytes of storage, costing {} shades",
            immutable_storage_requested,
            cost_of_storage
        );

        if cost_of_storage <= new_balance {
            msg!("User's stake account contains enough funds to cover immutable fee");

            // Compute amount to return to user + amount to transfer to emissions wallet
            let return_amount = new_balance.checked_sub(cost_of_storage).unwrap();
            msg!(
                "Returning {} shades to user, {} to emissions wallet",
                return_amount,
                cost_of_storage,
            );

            // Sanity check
            require_eq!(
                new_balance,
                cost_of_storage.checked_add(return_amount).unwrap(),
                ErrorCodes::InvalidTokenTransferAmounts
            );

            // Transfer to user, if necessary
            if return_amount > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new_with_signer(
                        self.accounts.token_program.to_account_info(),
                        anchor_spl::token::Transfer {
                            from: self.accounts.stake_account.to_account_info(),
                            to: self.accounts.owner_ata.to_account_info(),
                            authority: self.accounts.storage_config.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    return_amount,
                )
                .map_err(|_| ErrorCodes::FailedToReturnUserFunds)?;
            }

            // Transfer to emissions wallet
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
                cost_of_storage,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWallet)?;
        } else {
            let additional_shades: u64 = cost_of_storage.checked_sub(new_balance).unwrap();
            msg!(
                "User's stake account does not contain enough funds, charging user additional {} shades",
                additional_shades
            );

            // Sanity check
            require_eq!(
                cost_of_storage,
                additional_shades.checked_add(new_balance).unwrap(),
                ErrorCodes::InvalidTokenTransferAmounts
            );

            // Transfer from user
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(
                    self.accounts.token_program.to_account_info(),
                    anchor_spl::token::Transfer {
                        from: self.accounts.owner_ata.to_account_info(),
                        to: self.accounts.emissions_wallet.to_account_info(),
                        authority: self.accounts.owner.to_account_info(),
                    },
                    signer_seeds,
                ),
                additional_shades,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWalletFromUser)?;

            // Transfer to emissions wallet
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
                new_balance,
            )
            .map_err(|_| ErrorCodes::FailedToTransferToEmissionsWallet)?;
        }


        // Close account after having emptied it
        anchor_spl::token::close_account(CpiContext::new_with_signer(
            self.accounts.token_program.to_account_info(),
            anchor_spl::token::CloseAccount {
                account: self.accounts.stake_account.to_account_info(),
                destination: self.accounts.emissions_wallet.to_account_info(),
                authority: self.accounts.storage_config.to_account_info(),
            },
            signer_seeds,
        ))
        .map_err(|_| ErrorCodes::FailedToCloseAccount)?;
        
        Ok(())
    }
    fn get_identifier(&self) -> String {
        self.accounts.storage_account.identifier.clone()
    }
    fn mark_immutable(&mut self) {
        self.accounts.storage_account.immutable = true;
    }
}
