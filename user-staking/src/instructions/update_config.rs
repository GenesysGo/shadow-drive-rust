use crate::constants::admin1;
use crate::errors::ErrorCodes;
use crate::instructions::initialize_config::StorageConfig;
use anchor_lang::prelude::*;

/// This is the function that handles the `update_config` ix
pub fn handler(
    ctx: Context<UpdateConfig>,
    new_storage_cost: Option<u64>,
    new_storage_available: Option<u128>,
    new_admin_2: Option<Pubkey>,
    new_max_acct_size: Option<u64>,
    new_min_acct_size: Option<u64>,
) -> Result<()> {
    msg!("Updating StorageConfig");
    {
        let storage_config = &mut ctx.accounts.storage_config;

        // Update storage cost
        if let Some(storage_cost) = new_storage_cost {
            storage_config.shades_per_gib = storage_cost;
        }

        // Update storage available
        if let Some(storage_available) = new_storage_available {
            storage_config.storage_available = storage_available;
        }

        // Update admins. admin_1 is a program constant.
        if let Some(admin_2) = new_admin_2 {
            require!(
                ctx.accounts.admin.key() == admin1::ID,
                ErrorCodes::OnlyAdmin1CanChangeAdmins
            );
            storage_config.admin_2 = admin_2;
        }

        // Update account size limits
        if let Some(max_account_size) = new_max_acct_size {
            storage_config.max_account_size = max_account_size;
        }
        if let Some(min_account_size) = new_min_acct_size {
            storage_config.min_account_size = min_account_size;
        }
    }

    Ok(())
}

#[derive(Accounts)]
/// This `UpdateConfig` context is used to update the account which stores Shadow Drive
/// configuration data including storage costs, admin pubkeys.
pub struct UpdateConfig<'info> {
    /// This account is a PDA that holds storage config parameters and admin pubkeys.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// This account is the SPL's staking policy admin.
    /// Must be either freeze or mint authority
    #[account(mut, constraint=is_admin(&admin, &storage_config))]
    pub admin: Signer<'info>,
}

pub fn is_admin<'info>(
    admin: &Signer<'info>,
    storage_config: &Account<'info, StorageConfig>,
) -> bool {
    admin.key() == admin1::ID || admin.key() == storage_config.admin_2
}
