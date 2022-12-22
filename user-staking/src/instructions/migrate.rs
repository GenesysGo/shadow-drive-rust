use anchor_lang::prelude::*;
use crate::instructions::initialize_account::*;
use std::ops::{Deref, DerefMut};
use std::mem::take;

pub fn step1_handler(
    ctx: Context<MigrateStep1>
) -> Result<()> {

    // Migrate all data into migration pda
    ctx.accounts.migration.set_inner(ctx.accounts.storage_account.deref().clone());

    Ok(())
}

pub fn step2_handler(
    ctx: Context<MigrateStep2>
) -> Result<()> {

    // Migrate all data into migration pda
    let migration: &mut StorageAccount = ctx.accounts.migration.deref_mut();
    ctx.accounts.storage_account.set_inner(
        StorageAccountV2 {
            immutable: migration.immutable,
            to_be_deleted: migration.to_be_deleted,
            delete_request_epoch: migration.delete_request_epoch,
            storage: migration.storage,
            owner_1: migration.owner_1,
            account_counter_seed: migration.account_counter_seed,
            creation_time: migration.creation_time,
            creation_epoch: migration.creation_epoch,
            last_fee_epoch: migration.last_fee_epoch,
            identifier: take(&mut migration.identifier),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct MigrateStep1<'info> {

    /// Account to be migrated
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
    pub storage_account: Account<'info, StorageAccount>,

    /// Migration helper PDA
    #[account(
        init,
        space = calc_v1_storage(&storage_account.identifier),
        payer = owner,
        seeds = [
            "migration-helper".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub migration: Account<'info, StorageAccount>,

    /// User that is migrating
    #[account(mut, constraint = storage_account.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MigrateStep2<'info> {

    /// New account
    #[account(
        init,
        space = calc_v2_storage(&migration.identifier),
        payer = owner,
        seeds = [
            "storage-account".as_bytes(),
            &migration.owner_1.key().to_bytes(),
            &migration.account_counter_seed.to_le_bytes()
        ],
        bump,
    )]
    pub storage_account: Account<'info, StorageAccountV2>,

    /// Migration helper PDA
    #[account(
        mut,
        close = owner,
        seeds = [
            "migration-helper".as_bytes(),
            &storage_account.key().to_bytes(),
        ],
        bump,
    )]
    pub migration: Account<'info, StorageAccount>,

    /// User that is migrating
    #[account(mut, constraint = migration.is_owner(owner.key()))]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}