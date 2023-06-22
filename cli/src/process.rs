use super::Command;
use solana_sdk::signature::Signer;

impl Command {
    pub async fn process<T: Signer>(
        &self,
        signer: &T,
        client_signer: T,
        rpc_url: &str,
        skip_confirm: bool,
        auth: Option<String>,
    ) -> anyhow::Result<()> {
        println!();
        match self {
            Command::DriveCommand(drive_command) => {
                drive_command
                    .process(signer, client_signer, rpc_url, skip_confirm, auth)
                    .await
            }

            Command::NftCommand(nft_command) => {
                nft_command.process(signer, client_signer, rpc_url).await
            }
        }
    }
}
