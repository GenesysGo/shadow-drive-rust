use clap::Parser;
use shadow_drive_sdk::Signer;

pub(crate) mod init;

#[derive(Debug, Parser)]
pub enum CreatorGroupCommand {
    Init,
}

impl CreatorGroupCommand {
    pub async fn process(&self, signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
        match self {
            // Initialize a creator group
            CreatorGroupCommand::Init => init::process(signer, rpc_url)
                .await
                .map(|_creator_group_initialized| {}),
        }
    }
}
