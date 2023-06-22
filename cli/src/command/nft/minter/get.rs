use shadow_drive_sdk::Pubkey;
use shadowy_super_minter::state::{file_type::AccountDeserialize, ShadowySuperMinter};
use solana_client::rpc_client::RpcClient;

pub(crate) async fn process(minter: &Pubkey, rpc_url: &str) -> Result<(), anyhow::Error> {
    let client = RpcClient::new(rpc_url);

    let Ok(minter_data) = client.get_account_data(minter) else {
        return Err(anyhow::Error::msg(format!("No account found at {minter}")))
    };

    let mut minter_data_cursor = minter_data.as_slice();
    let Ok(ssm) = ShadowySuperMinter::try_deserialize(&mut minter_data_cursor) else {
        return Err(anyhow::Error::msg(format!("Failed to deserialize ShadowySuperMinter")))

    };
    println!("{ssm:#?}");

    Ok(())
}
