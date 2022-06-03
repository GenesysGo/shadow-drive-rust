use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

mod create_storage_account;
mod get_storage_account;
mod upload_file;

pub use create_storage_account::*;
pub use get_storage_account::*;
pub use upload_file::*;

use crate::{derived_addresses, models::ShadowDriveResult};

pub struct Client<T>
where
    T: Signer,
{
    wallet: T,
    rpc_client: RpcClient,
    http_client: reqwest::Client,
    user_info: Option<Pubkey>,
}

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn new(wallet: T, rpc_client: RpcClient) -> ShadowDriveResult<Self> {
        let mut result = Self {
            wallet,
            rpc_client,
            http_client: reqwest::Client::new(),
            user_info: None,
        };
        let wallet_pubkey = result.wallet.pubkey();
        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);
        if let Ok(_) = result.rpc_client.get_account(&user_info) {
            result.user_info = Some(user_info)
        }

        Ok(result)
    }
}
