use clap::Parser;
use shadow_drive_sdk::Signer;

use self::{collection::*, creator_group::*, minter::*};

pub mod collection;
pub mod creator_group;
pub mod minter;
pub mod utils;

#[derive(Debug, Parser)]
pub enum NftCommand {
    /// Commands for creating and managing shadow nft minters.
    #[clap(subcommand)]
    Minter(MinterCommand),

    /// Commands for creating and managing creator groups.
    #[clap(subcommand)]
    CreatorGroup(CreatorGroupCommand),

    /// Commands for creating and managing collections.
    #[clap(subcommand)]
    Collection(CollectionCommand),
}

impl NftCommand {
    pub async fn process<T: Signer>(
        &self,
        signer: &T,
        client_signer: T,
        rpc_url: &str,
    ) -> anyhow::Result<()> {
        match self {
            NftCommand::Minter(minter_cmd) => {
                minter_cmd.process(signer, client_signer, rpc_url).await
            }
            NftCommand::CreatorGroup(group_command) => group_command.process(signer, rpc_url).await,
            NftCommand::Collection(collection_command) => {
                collection_command.process(signer, rpc_url).await
            }
        }
    }
}
