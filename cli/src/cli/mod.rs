pub mod process;

use byte_unit::Byte;
use clap::Parser;
use shadow_drive_cli::{parse_filesize, pubkey_arg};
use shadow_drive_cli::FILE_UPLOAD_BATCH_SIZE;
use solana_sdk::pubkey::Pubkey;

/// Manually specify a cluster url and/or keypair.
/// Those values otherwise default to the Solana CLI config file.
/// All values other than `url` and `keypair` exist only to satisfy compatibility
/// with keypair resolution.
#[derive(Debug, Parser)]
pub struct ConfigOverride {
    /// The target URL for the cluster. See Solana CLI documentation on how to use this.
    /// Default values and usage patterns are identical to Solana CLI.
    #[clap(short, long)]
    pub url: Option<String>,
    /// The target signer for transactions. See Solana CLI documentation on how to use this.
    /// Default values and usage patterns are identical to Solana CLI.
    #[clap(short, long)]
    pub keypair: Option<String>,
    // The CLI options listed below are needed to resolve certain signer paths
    /// Skip BIP-39 seed phrase validation (not recommended)
    #[clap(long, name = "skip_seed_phrase_validation")]
    pub skip_seed_phrase_validation: bool,
    /// Manually confirm the signer before proceeding
    #[clap(long, name = "confirm_key")]
    pub confirm_key: bool,
    /// Bypass the manual confirmation on various operations that
    /// alter the state of the storage network.
    #[clap(long)]
    pub skip_confirm: bool,
    /// Supply a JWT to be included as a Bearer auth token to each RPC request.
    /// Use keyword "genesysgo" to automatically
    /// authenticate with a GenesysGo Premium RPC endpoint.
    /// GenesysGo Account ID is inferred from `-u/--url` path.
    /// See also the `shadow-rpc-auth` subcommand for manually
    /// acquiring an auth token.
    #[clap(long)]
    pub auth: Option<String>,
}

/// Perform Shadow Drive operations on the command-line.
/// This CLI is written in Rust, and conforms to the interfaces
/// for signer specification available in the official
/// Solana command-line binaries such as `solana` and `spl-token`.
#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(flatten)]
    pub cfg_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    ShadowRpcAuth,
    /// Create an account on which to store data.
    /// Storage accounts can be globally, irreversibly marked immutable
    /// for a one-time fee.
    /// Otherwise, files can be added or deleted from them, and space
    /// rented indefinitely.
    CreateStorageAccount {
        /// Unique identifier for your storage account
        name: String,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Queues a storage account for deletion. While the request is
    /// still enqueued and not yet carried out, a cancellation
    /// can be made (see cancel-delete-storage-account subcommand).
    DeleteStorageAccount {
        /// The account to delete
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Cancels the deletion of a storage account enqueued for deletion.
    CancelDeleteStorageAccount {
        /// The account for which to cancel deletion.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Redeem tokens afforded to a storage account after reducing storage capacity.
    ClaimStake {
        /// The account whose stake to claim.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Increase the capacity of a storage account.
    AddStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Increase the immutable storage capacity of a storage account.
    AddImmutableStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Reduce the capacity of a storage account.
    ReduceStorage {
        /// Storage account to modify
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// File size string, accepts KB, MB, GB, e.g. "10MB"
        #[clap(parse(try_from_str = parse_filesize))]
        size: Byte,
    },
    /// Make a storage account immutable. This is irreversible.
    MakeStorageImmutable {
        /// Storage account to be marked immutable
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Fetch the metadata pertaining to a storage account.
    GetStorageAccount {
        /// Account whose metadata will be fetched.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Fetch a list of storage accounts owned by a particular pubkey.
    /// If no owner is provided, the configured signer is used.
    GetStorageAccounts {
        /// Searches for storage accounts owned by this owner.
        #[clap(parse(try_from_str = pubkey_arg))]
        owner: Option<Pubkey>,
    },
    /// List all the files in a storage account.
    ListFiles {
        /// Storage account whose files to list.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
    },
    /// Get a file, assume it's text, and print it.
    GetText {
        /// Storage account where the file is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to fetch
        file: String,
    },
    /// Get basic file object data from a storage account file.
    GetObjectData {
        /// Storage account where the file is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to examine.
        file: String,
    },
    /// Delete a file from a storage account.
    DeleteFile {
        /// Storage account where the file to delete is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Name of the file to delete.
        file: String,
    },
    /// Has to be the same name as a previously uploaded file
    EditFile {
        /// Storage account where the file to edit is located.
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// Path to the new version of the file. Must be the same
        /// name as the file you are editing.
        file: String,
    },
    /// Upload one or more files to a storage account.
    StoreFiles {
        /// Batch size for file uploads, default 100, only relevant for large
        /// numbers of uploads
        #[clap(long, default_value_t=FILE_UPLOAD_BATCH_SIZE)]
        batch_size: usize,
        /// The storage account on which to upload the files
        #[clap(parse(try_from_str = pubkey_arg))]
        storage_account: Pubkey,
        /// A list of one or more filepaths, each of which is to be uploaded.
        #[clap(min_values = 1)]
        files: Vec<String>,
    },
}
