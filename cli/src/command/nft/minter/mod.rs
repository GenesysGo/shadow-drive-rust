use clap::Parser;

use shadow_drive_sdk::{Pubkey, Signer};

mod get;
mod init;
mod mint;

#[derive(Debug, Parser)]
pub enum MinterCommand {
    /// Initializes a minter given a creator_group and collection
    Init,

    /// Gets a minter from the chain and prints its state
    Get { minter: Pubkey },

    /// Mints an nft from the provided minter
    Mint { minter: Pubkey },
}

impl MinterCommand {
    pub async fn process(
        &self,
        signer: &impl Signer,
        client_signer: impl Signer,
        rpc_url: &str,
    ) -> anyhow::Result<()> {
        match self {
            MinterCommand::Init => init::process(signer, client_signer, rpc_url).await,

            MinterCommand::Get { minter } => get::process(minter, rpc_url).await,

            MinterCommand::Mint { minter } => mint::process(signer, *minter, rpc_url).await,
        }
    }
}
