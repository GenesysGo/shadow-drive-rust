use clap::Parser;
use shadow_drive_sdk::Signer;

mod init;

#[derive(Debug, Parser)]
pub enum CollectionCommand {
    Init,
}
impl CollectionCommand {
    pub async fn process(&self, signer: &impl Signer, rpc_url: &str) -> anyhow::Result<()> {
        match self {
            CollectionCommand::Init => init::process(signer, rpc_url).await,
        }
    }
}
