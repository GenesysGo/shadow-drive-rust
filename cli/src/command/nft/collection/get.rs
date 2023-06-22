use shadow_drive_sdk::Pubkey;
use shadow_nft_standard::common::collection::Collection;
use shadowy_super_minter::state::file_type::AccountDeserialize;
use solana_client::rpc_client::RpcClient;

pub(crate) async fn process(collection: &Pubkey, rpc_url: &str) -> Result<(), anyhow::Error> {
    let client = RpcClient::new(rpc_url);

    let Ok(collection_data) = client.get_account_data(collection) else {
        return Err(anyhow::Error::msg(format!("No account found at {collection}")))
    };

    let mut collection_data_cursor = collection_data.as_slice();
    let Ok(ssm) = Collection::try_deserialize(&mut collection_data_cursor) else {
        return Err(anyhow::Error::msg(format!("Failed to deserialize Collection")))

    };
    println!("{ssm:#?}");

    Ok(())
}
