use crate::{
    constants::TOKEN_MINT,
    derived_addresses,
    models::{ShadowDriveResult, ShdwDriveResponse},
    ShadowDriveClient,
};
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_token::instruction::transfer;

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    ///  Allows user to top up stake account, transfering some amount of $SHDW.
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that you want to top up stake for.
    ///  * `amount` - The amount of $SHDW to transfer into stake account
    /// # Example
    ///
    /// ```
    /// # use shadow_drive_rust::{ShadowDriveClient, derived_addresses::storage_account};
    /// # use solana_client::rpc_client::RpcClient;
    /// # use solana_sdk::{
    /// # pubkey::Pubkey,
    /// # signature::Keypair,
    /// # signer::{keypair::read_keypair_file, Signer},
    /// # };
    /// #
    /// # let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    /// # let user_pubkey = keypair.pubkey();
    /// # let rpc_client = RpcClient::new("https://ssc-dao.genesysgo.net");
    /// # let shdw_drive_client = ShadowDriveClient::new(keypair, rpc_client);
    /// # let (storage_account_key, _) = storage_account(&user_pubkey, 0);
    /// # let top_up_amount: u64 = 1000
    /// #
    /// let top_up = shdw_drive_client
    ///     .top_up(&storage_account_key, top_up_amount)
    ///     .await?;
    /// ```
    pub async fn top_up(
        &self,
        storage_account_key: &Pubkey,
        amount: u64,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet_pubkey = self.wallet.pubkey();
        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);
        let (stake_account, _) = derived_addresses::stake_account(storage_account_key);

        let instruction = transfer(
            &spl_token::id(),
            &owner_ata,
            &stake_account,
            &self.wallet.pubkey(),
            &[&self.wallet.pubkey()],
            amount,
        )
        .unwrap();

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        let recent_blockhash = self.rpc_client.get_latest_blockhash().await?;
        txn.try_sign(&[&self.wallet], recent_blockhash)?;
        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn).await?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
}
