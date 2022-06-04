use solana_client::rpc_client::RpcClient;
use solana_sdk::signer::Signer;

mod add_storage;
mod create_storage_account;
mod delete_file;
mod delete_storage_account;
mod get_storage_account;
mod list_objects;
mod reduce_storage;
mod upload_file;

pub use add_storage::*;
pub use create_storage_account::*;
pub use delete_file::*;
pub use delete_storage_account::*;
pub use get_storage_account::*;
pub use list_objects::*;
pub use reduce_storage::*;
pub use upload_file::*;

pub struct Client<T>
where
    T: Signer,
{
    wallet: T,
    rpc_client: RpcClient,
    http_client: reqwest::Client,
}

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub fn new(wallet: T, rpc_client: RpcClient) -> Self {
        Self {
            wallet,
            rpc_client,
            http_client: reqwest::Client::new(),
        }
    }
}
