use crate::{
    errors::ErrorCodes,
    instructions::{initialize_config::StorageConfig, update_config::is_admin},
};
use anchor_lang::prelude::*;
use std::convert::TryInto;

/// This is the function that handles the `mutable_fees` ix
pub fn handler(
    ctx: Context<MutableFees>,
    shades_per_gib_per_epoch: Option<u64>,
    crank_bps: Option<u32>,
) -> Result<()> {
    let storage_config = &mut ctx.accounts.storage_config;
    // We should assume if some value is passed in for both shades_per_gib_per_epoch and
    // crank_bps, then we can assume fees should be enabled or updated
    if shades_per_gib_per_epoch.is_some() && crank_bps.is_some() {
        // Mutable fees are off. Turn them on.
        msg!(
            "Turning on mutable account storage fees. Shades/GB = {}; Crank bps = {}",
            shades_per_gib_per_epoch.unwrap(),
            crank_bps.unwrap()
        );

        let clock = Clock::get()?;
        storage_config.mutable_fee_start_epoch = Some(clock.epoch.try_into().unwrap());
        storage_config.shades_per_gib_per_epoch = shades_per_gib_per_epoch.unwrap();
        storage_config.crank_bps = crank_bps.unwrap().try_into().unwrap();
    } else if !shades_per_gib_per_epoch.is_some() && !crank_bps.is_some() {
        // Mutable fees are on. Turn them off.
        msg!("Turning off mutable account storage fees");
        storage_config.mutable_fee_start_epoch = None;
        storage_config.shades_per_gib_per_epoch = 0;
        storage_config.crank_bps = 0;
    } else {
        // Check for valid fee parameters and return the appropriate error
        require!(shades_per_gib_per_epoch.is_some(), ErrorCodes::NeedSomeFees);
        require!(crank_bps.is_some(), ErrorCodes::NeedSomeCrankBps);
    }

    Ok(())
}

#[derive(Accounts)]
/// This `MutableFees` context is used to initialize the account which stores Shadow Drive
/// configuration data including storage costs, admin pubkeys.
pub struct MutableFees<'info> {
    /// This account is a PDA that holds the SPL's staking and slashing policy.
    /// This is the account that signs transactions on behalf of the program to
    /// distribute staking rewards.
    #[account(
        mut,
        seeds = [
            "storage-config".as_bytes()
        ],
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// This account is the SPL's staking policy admin.
    /// Must be either freeze or mint authority
    #[account(mut, constraint = is_admin(&admin, &storage_config))]
    pub admin: Signer<'info>,
}
