use clap::Parser;

use shadow_drive_sdk::{Pubkey, Signer};

mod get;
mod init;

#[derive(Debug, Parser)]
pub enum MinterCommand {
    Init,
    Get { minter: Pubkey },
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
        }
    }
}
