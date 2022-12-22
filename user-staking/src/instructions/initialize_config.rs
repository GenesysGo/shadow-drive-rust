use crate::constants::*;
use crate::errors::ErrorCodes;
use anchor_lang::prelude::*;

/// This is the function that handles the `initialize_config` ix
pub fn handler(
    ctx: Context<InitializeStorageConfig>,
    uploader: Pubkey,
    admin_2: Option<Pubkey>,
    // admin_3: Option<Pubkey>,
    // admin_4: Option<Pubkey>,
) -> Result<()> {
    msg!("Initializing StorageConfig");
    {
        let storage_config = &mut ctx.accounts.storage_config;

        // Initial storage cost
        storage_config.shades_per_gib = INITIAL_STORAGE_COST;

        // Initial storage available
        storage_config.storage_available = INITIAL_STORAGE_AVAILABLE;

        // Populate admins. admin_1 is a program constant.
        storage_config.admin_2 = admin_2.unwrap_or(ctx.accounts.admin_1.key());
        // storage_config.admin_3 = admin_3.unwrap_or(ctx.accounts.admin_1.key());
        // storage_config.admin_4 = admin_4.unwrap_or(ctx.accounts.admin_1.key());

        // Store uploader pubkey
        storage_config.uploader = uploader;

        // Initialize mutable fee variables
        storage_config.mutable_fee_start_epoch = None;
        storage_config.shades_per_gib_per_epoch = 0;
        storage_config.crank_bps = INITIAL_CRANK_FEE_BPS;

        // Initialize account limits
        storage_config.max_account_size = MAX_ACCOUNT_SIZE;
        storage_config.min_account_size = MIN_ACCOUNT_SIZE;
    }

    Ok(())
}

#[derive(Accounts)]
/// This `InitializeStorageConfig` context is used to initialize the account which stores Shadow Drive
/// configuration data including storage costs, admin pubkeys.
pub struct InitializeStorageConfig<'info> {
    /// This account is a PDA that holds the SPL's staking and slashing policy.
    /// This is the account that signs transactions on behalf of the program to
    /// distribute staking rewards.
    #[account(
        init,
        payer = admin_1,
        seeds = [
            "storage-config".as_bytes()
        ],
        space = std::mem::size_of::<StorageConfig>() + 4, // 4 extra for None --> Some(u32)
        bump,
    )]
    pub storage_config: Box<Account<'info, StorageConfig>>,

    /// This account is the SPL's staking policy admin.
    /// Must be either freeze or mint authority
    #[account(mut, address=crate::constants::admin1::ID)]
    pub admin_1: Signer<'info>,

    /// System Program
    pub system_program: Program<'info, System>,

    /// Rent Program
    pub rent: Sysvar<'info, Rent>,
}

#[account]
pub struct StorageConfig {
    /// Storage costs in shades per GiB
    pub shades_per_gib: u64,

    /// Total storage available (or remaining)
    pub storage_available: u128,

    /// Pubkey of SHDW token account that holds storage fees/stake
    pub token_account: Pubkey,

    /// Optional Admin 2
    pub admin_2: Pubkey,

    // /// Optional Admin 3
    // pub admin_3: Pubkey,

    // /// Optional Admin 4
    // pub admin_4: Pubkey,
    /// Uploader key, used to sign off on successful storage + CSAM scan
    pub uploader: Pubkey,

    /// Epoch at which mutable_account_fees turned on
    pub mutable_fee_start_epoch: Option<u32>,

    /// Mutable fee rate
    pub shades_per_gib_per_epoch: u64,

    /// Basis points cranker gets from cranking
    pub crank_bps: u16,

    /// Maximum size of a storage account
    pub max_account_size: u64,

    /// Minimum size of a storage account
    pub min_account_size: u64,
}

impl StorageConfig {
    pub fn validate_storage(&self, storage: u64) -> Result<u64> {
        if storage <= self.max_account_size && storage >= self.min_account_size {
            Ok(storage)
        } else if storage < self.min_account_size {
            msg!("Tiny account, failing");
            Err(ErrorCodes::AccountTooSmall.into())
        } else {
            msg!("Very large account, failing");
            Err(ErrorCodes::ExceededStorageLimit.into())
        }
    }
}
