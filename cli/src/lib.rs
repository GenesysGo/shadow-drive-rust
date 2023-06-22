pub mod command;
pub mod process;

pub mod state;
pub mod utils;

use clap::Parser;

use command::{drive::*, nft::*};

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
    /// Commands for creating and managing shadow drive accounts and files
    #[clap(subcommand, name = "drive")]
    DriveCommand(DriveCommand),

    /// Commands for creating and managing shadow nft minters and metadata accounts
    #[clap(subcommand, name = "nft")]
    NftCommand(NftCommand),
}
