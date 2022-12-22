use anchor_lang::prelude::*;

declare_id!("2e1wdyNhUvE76y6yUCvah2KaviavMJYKoRun8acMRBZZ");

pub mod constants;
pub mod errors;
pub mod instructions;

use instructions::{
    bad_csam::*, claim_stake::*, crank::*, decrease_storage::*, delete_account::*,
    increase_storage::*, initialize_account::*, initialize_config::*, make_account_immutable::*,
    migrate::*, mutable_fees::*, redeem_rent::*, refresh_stake::*, request_delete_account::*,
    unmark_delete_account::*, update_account::*, update_config::*,
};
#[program]
pub mod shadow_drive_user_staking {

    use super::*;

    /// Context: This is for admin use. This is to be called first, as this initializes Shadow Drive access on-chain!
    /// Function: The primary function of this is to initialize an account that stores the configuration/parameters of the storage program on-chain, e.g. admin pubkeys, storage cost.
    pub fn initialize_config(
        ctx: Context<InitializeStorageConfig>,
        uploader: Pubkey,
        admin_2: Option<Pubkey>,
        // admin_3: Option<Pubkey>,
        // admin_4: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_config::handler(ctx, uploader, admin_2) //, admin_3, admin_4)
    }

    /// Context: This is for admin use.
    /// Function: The primary function of this is update the storage_config account which stores Shadow Drive parameters on-chain, e.g. admin pubkeys, storage cost.
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_storage_cost: Option<u64>,
        new_storage_available: Option<u128>,
        new_admin_2: Option<Pubkey>,
        // new_admin_3: Option<Pubkey>,
        // new_admin_4: Option<Pubkey>,
        new_max_acct_size: Option<u64>,
        new_min_acct_size: Option<u64>,
    ) -> Result<()> {
        instructions::update_config::handler(
            ctx,
            new_storage_cost,
            new_storage_available,
            new_admin_2,
            new_max_acct_size,
            new_min_acct_size,
        )
    }

    /// Context: This is for admin use.
    /// Function: The primary function of this is to toggle fees for mutable account storage on and off.
    pub fn mutable_fees(
        ctx: Context<MutableFees>,
        shades_per_gb_per_epoch: Option<u64>,
        crank_bps: Option<u32>,
    ) -> Result<()> {
        instructions::mutable_fees::handler(ctx, shades_per_gb_per_epoch, crank_bps)
    }

    /// Context: This is user-facing. This is to be done whenever the user decides.
    /// Function: This allows the user to initialize a storage account with some specified number of bytes.
    pub fn initialize_account(
        ctx: Context<InitializeStorageAccountV1>,
        identifier: String,
        storage: u64,
        owner_2: Option<Pubkey>,
        // owner_3: Option<Pubkey>,
        // owner_4: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_account::handler(ctx, identifier, storage, owner_2)
        //, owner_3, owner_4)
    }

    /// Context: This is user-facing. This is to be done whenever the user decides.
    /// Function: This allows the user to initialize a storage account with some specified number of bytes.
    pub fn initialize_account2(
        ctx: Context<InitializeStorageAccountV2>,
        identifier: String,
        storage: u64,
        // owner_2: Option<Pubkey>,
        // owner_3: Option<Pubkey>,
        // owner_4: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_account::handler(ctx, identifier, storage, None)
        //, owner_3, owner_4)
    }

    /// Context: This is user-facing. This is to be done whenever the user decides.
    /// Function: This allows the user to change the amount of storage they have for this storage account.
    pub fn update_account(
        ctx: Context<UpdateAccountV1>,
        identifier: Option<String>,
        owner_2: Option<Pubkey>,
        // owner_3: Option<Pubkey>,
        // owner_4: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_account::handler(ctx, identifier, owner_2) //, owner_3, owner_4)
    }

    /// Context: This is user-facing. This is to be done whenever the user decides.
    /// Function: This allows the user to change the amount of storage they have for this storage account.
    pub fn update_account2(
        ctx: Context<UpdateAccountV2>,
        identifier: Option<String>,
        // owner_2: Option<Pubkey>,
        // owner_3: Option<Pubkey>,
        // owner_4: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_account::handler(ctx, identifier, None) //, owner_3, owner_4)
    }

    /// Context: This is user-facing. This is to be done after our upload server verifies all is well.
    /// Function: This stores the file metadata + location on-chain.
    // pub fn store_file(
    //     ctx: Context<StoreFile>,
    //     filename: String,
    //     //url: String,
    //     sha256_hash: String,
    //     // created: i64,
    //     size: u64,
    // ) -> Result<()> {
    //     instructions::store_file::handler(ctx, filename, size, sha256_hash)
    // }

    /// Context: This is user-facing, but requires our uploader's signature. This is to be done after our upload server verifies all is well.
    /// Function: This updates the file metadata on-chain upon user edits.
    // pub fn edit_file(ctx: Context<EditFile>, sha256_hash: String, size: u64) -> Result<()> {
    //     instructions::edit_file::handler(ctx, size, sha256_hash)
    // }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and records the request time. Fails if parent account is marked as immutable.
    // pub fn request_delete_file(ctx: Context<RequestDeleteFile>) -> Result<()> {
    //     instructions::request_delete_file::handler(ctx)
    // }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and records the request time. Fails if account is marked as immutable.
    pub fn request_delete_account(ctx: Context<RequestDeleteAccountV1>) -> Result<()> {
        instructions::request_delete_account::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and records the request time. Fails if account is marked as immutable.
    pub fn request_delete_account2(ctx: Context<RequestDeleteAccountV2>) -> Result<()> {
        instructions::request_delete_account::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and resets the request time. Fails if parent account is marked as immutable.
    // pub fn unmark_delete_file(ctx: Context<UnmarkDeleteFile>) -> Result<()> {
    //     instructions::unmark_delete_file::handler(ctx)
    // }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and resets the request time. Fails if account is marked as immutable.
    pub fn unmark_delete_account(ctx: Context<UnmarkDeleteAccountV1>) -> Result<()> {
        instructions::unmark_delete_account::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: This updates a boolean flag and resets the request time. Fails if account is marked as immutable.
    pub fn unmark_delete_account2(ctx: Context<UnmarkDeleteAccountV2>) -> Result<()> {
        instructions::unmark_delete_account::handler(ctx)
    }

    /// Context: This is for admin use.
    /// Function: This deletes the corresponding `File` account and updates storage available in user's storage account.
    /// Fails if file is marked as immutable, or if time elapsed since request is less than the grace period.
    // pub fn delete_file(ctx: Context<DeleteFile>) -> Result<()> {
    //     instructions::delete_file::handler(ctx)
    // }

    /// Context: This is user-facing.
    /// Function: This deletes the corresponding `File` account, allowing user to redeem SOL rent in v1.5
    pub fn redeem_rent(ctx: Context<RedeemRent>) -> Result<()> {
        instructions::redeem_rent::handler(ctx)
    }

    /// Context: This is for admin use.
    /// Function: This deletes the corresponding `StorageAccount` account and return's user funds.
    /// Fails if file is marked as immutable, or if time elapsed since request is less than the grace period.
    pub fn delete_account(ctx: Context<DeleteAccountV1>) -> Result<()> {
        instructions::delete_account::handler(ctx)
    }

    /// Context: This is for admin use.
    /// Function: This deletes the corresponding `StorageAccount` account and return's user funds.
    /// Fails if file is marked as immutable, or if time elapsed since request is less than the grace period.
    pub fn delete_account2(ctx: Context<DeleteAccountV2>) -> Result<()> {
        instructions::delete_account::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: This marks the corresponding `StorageAccount` account as immutable,
    /// and transfers all funds from `stake_account` to operator emissions wallet.
    pub fn make_account_immutable(ctx: Context<MakeAccountImmutableV1>) -> Result<()> {
        instructions::make_account_immutable::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: This marks the corresponding `StorageAccount` account as immutable,
    /// and transfers all funds from `stake_account` to operator emissions wallet.
    pub fn make_account_immutable2(ctx: Context<MakeAccountImmutableV2>) -> Result<()> {
        instructions::make_account_immutable::handler(ctx)
    }

    /// Context: This is for admin use.
    /// Function: Upon a bad csam scan, rugs user,
    /// deleting storage account and transferring funds to emissions wallet
    pub fn bad_csam(ctx: Context<BadCsam1>, storage_available: u64) -> Result<()> {
        instructions::bad_csam::handler(ctx, storage_available)
    }

    /// Context: This is for admin use.
    /// Function: Upon a bad csam scan, rugs user,
    /// deleting storage account and transferring funds to emissions wallet
    pub fn bad_csam2(ctx: Context<BadCsam2>, storage_available: u64) -> Result<()> {
        instructions::bad_csam::handler(ctx, storage_available)
    }

    /// Context: This is user facing.
    /// Function: allows user to pay for more storage at current rate.
    pub fn increase_storage(
        ctx: Context<IncreaseStorageV1>,
        additional_storage: u64,
    ) -> Result<()> {
        instructions::increase_storage::handler(ctx, additional_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to pay for more storage at current rate.
    pub fn increase_storage2(
        ctx: Context<IncreaseStorageV2>,
        additional_storage: u64,
    ) -> Result<()> {
        instructions::increase_storage::handler(ctx, additional_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to pay for more storage at current rate, after having marked an account as immutable
    pub fn increase_immutable_storage(
        ctx: Context<IncreaseImmutableStorageV1>,
        additional_storage: u64,
    ) -> Result<()> {
        instructions::increase_storage::handler(ctx, additional_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to pay for more storage at current rate, after having marked an account as immutable
    pub fn increase_immutable_storage2(
        ctx: Context<IncreaseImmutableStorageV2>,
        additional_storage: u64,
    ) -> Result<()> {
        instructions::increase_storage::handler(ctx, additional_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to reduce storage, up to current available storage,
    /// and begins an unstake ticket.
    pub fn decrease_storage(ctx: Context<DecreaseStorageV1>, remove_storage: u64) -> Result<()> {
        instructions::decrease_storage::handler(ctx, remove_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to reduce storage, up to current available storage,
    /// and begins an unstake ticket.
    pub fn decrease_storage2(ctx: Context<DecreaseStorageV2>, remove_storage: u64) -> Result<()> {
        instructions::decrease_storage::handler(ctx, remove_storage)
    }

    /// Context: This is user facing.
    /// Function: allows user to claim stake from unstake ticket.
    /// Fails if user has not waited an appropriate amount of time.
    pub fn claim_stake(ctx: Context<ClaimStakeV1>) -> Result<()> {
        instructions::claim_stake::handler(ctx)
    }

    /// Context: This is user facing.
    /// Function: allows user to claim stake from unstake ticket.
    /// Fails if user has not waited an appropriate amount of time.
    pub fn claim_stake2(ctx: Context<ClaimStakeV2>) -> Result<()> {
        instructions::claim_stake::handler(ctx)
    }

    /// Context: This is a public function, callable by anyone.
    /// Function: collects fees from user stake account and
    /// sends it to the operator emissions wallet.
    pub fn crank(ctx: Context<CrankV1>) -> Result<()> {
        instructions::crank::handler(ctx)
    }

    /// Context: This is a public function, callable by anyone.
    /// Function: collects fees from user stake account and
    /// sends it to the operator emissions wallet.
    pub fn crank2(ctx: Context<CrankV2>) -> Result<()> {
        instructions::crank::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: allows user to top off stake account, and unmarks deletion.
    pub fn refresh_stake(ctx: Context<RefreshStakeV1>) -> Result<()> {
        instructions::refresh_stake::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: allows user to top off stake account, and unmarks deletion.
    pub fn refresh_stake2(ctx: Context<RefreshStakeV2>) -> Result<()> {
        instructions::refresh_stake::handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: allows user to top off stake account, and unmarks deletion.
    pub fn migrate_step1(ctx: Context<MigrateStep1>) -> Result<()> {
        instructions::migrate::step1_handler(ctx)
    }

    /// Context: This is user-facing.
    /// Function: allows user to top off stake account, and unmarks deletion.
    pub fn migrate_step2(ctx: Context<MigrateStep2>) -> Result<()> {
        instructions::migrate::step2_handler(ctx)
    }
}
