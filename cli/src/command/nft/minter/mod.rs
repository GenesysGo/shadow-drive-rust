use std::ops::Deref;

use clap::Parser;

use shadow_drive_sdk::Signer;

mod init;
#[derive(Debug, Parser)]
pub enum MinterCommand {
    Init,
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
        }
    }
}
