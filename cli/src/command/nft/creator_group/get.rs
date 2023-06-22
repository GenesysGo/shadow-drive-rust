use shadow_drive_sdk::Pubkey;
use shadow_nft_standard::common::creator_group::CreatorGroup;
use shadowy_super_minter::state::file_type::AccountDeserialize;
use solana_client::rpc_client::RpcClient;

pub(crate) async fn process(creator_group: &Pubkey, rpc_url: &str) -> Result<(), anyhow::Error> {
    let client = RpcClient::new(rpc_url);

    let Ok(creator_group_data) = client.get_account_data(creator_group) else {
        return Err(anyhow::Error::msg(format!("No account found at {creator_group}")))
    };

    let mut creator_group_data_cursor = creator_group_data.as_slice();
    let Ok(onchain_creator_group) = CreatorGroup::try_deserialize(&mut creator_group_data_cursor) else {
        return Err(anyhow::Error::msg(format!("Failed to deserialize CreatorGroup")))

    };
    println!("{onchain_creator_group:#?}");

    Ok(())
}
