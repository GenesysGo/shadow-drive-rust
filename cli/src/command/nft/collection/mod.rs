use clap::Parser;
use shadow_drive_sdk::{Pubkey, Signer};

mod get;
mod init;
mod withdraw;

#[derive(Debug, Parser)]
pub enum CollectionCommand {
    Init,
    Get { collection: Pubkey },
    Withdraw { collection: Pubkey },
}
impl CollectionCommand {
    pub async fn process(&self, signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
        match self {
            CollectionCommand::Init => init::process(signer, rpc_url).await,

            CollectionCommand::Get { collection } => get::process(collection, rpc_url).await,
            CollectionCommand::Withdraw { collection } => {
                withdraw::process(signer, *collection, rpc_url).await
            }
        }
    }
}
