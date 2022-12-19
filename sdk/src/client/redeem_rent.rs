use anchor_lang::{InstructionData, ToAccountMetas};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction as shdw_drive_instructions;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction,
};

use super::ShadowDriveClient;

use crate::{constants::PROGRAM_ADDRESS, models::*};

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Reclaims the Solana rent from any on-chain file accounts. Older versions of the Shadow Drive used to create accounts for uploaded files.
    ///
    /// * `storage_account_key` - The public key of the [`StorageAccount`](crate::models::StorageAccount) that contained the deleted file.
    /// * `file_account_key` - The public key of the File account to be closed.
    ///
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
    /// # use std::str::FromStr;
    /// #
    /// # let keypair = read_keypair_file(KEYPAIR_PATH).expect("failed to load keypair at path");
    /// # let user_pubkey = keypair.pubkey();
    /// # let rpc_client = RpcClient::new("https://ssc-dao.genesysgo.net");
    /// # let shdw_drive_client = ShadowDriveClient::new(keypair, rpc_client);
    /// # let (storage_account_key, _) = storage_account(&user_pubkey, 0);
    /// # let file_account_key =  Pubkey::from_str("ACbwxy6KEqLPKXBMbYXp48F8dPchbbKEcQEbmcCSZe31").unwrap();
    /// #
    ///let redeem_rent_response = shdw_drive_client
    ///     .redeem_rent(&storage_account_key, &file_account_key)
    ///     .await?;
    /// ```
    pub async fn redeem_rent(
        &self,
        storage_account_key: &Pubkey,
        file_account_key: &Pubkey,
    ) -> ShadowDriveResult<ShdwDriveResponse> {
        let wallet_pubkey = self.wallet.pubkey();

        let accounts = shdw_drive_accounts::RedeemRent {
            storage_account: *storage_account_key,
            file: *file_account_key,
            owner: wallet_pubkey,
        };

        let args = shdw_drive_instructions::RedeemRent {};

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));
        txn.try_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;
        let txn_result = self.rpc_client.send_and_confirm_transaction(&txn).await?;

        Ok(ShdwDriveResponse {
            txid: txn_result.to_string(),
        })
    }
}
