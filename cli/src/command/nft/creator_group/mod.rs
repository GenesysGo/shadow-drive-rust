use clap::Parser;
use shadow_drive_sdk::{Pubkey, Signer};

pub(crate) mod get;
pub(crate) mod init;

#[derive(Debug, Parser)]
pub enum CreatorGroupCommand {
    Init,
    Get { creator_group: Pubkey },
}

impl CreatorGroupCommand {
    pub async fn process(&self, signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
        match self {
            // Initialize a creator group
            CreatorGroupCommand::Init => init::process(signer, rpc_url)
                .await
                .map(|_creator_group_initialized| {}),

            CreatorGroupCommand::Get { creator_group } => {
                get::process(creator_group, rpc_url).await
            }
        }
    }
}
