use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use byte_unit::Byte;
use serde_json::{json, Value};
use shadow_drive_user_staking::accounts as shdw_drive_accounts;
use shadow_drive_user_staking::instruction::InitializeAccount;
use shadow_drive_user_staking::instructions::initialize_account::UserInfo;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_client::serialize_and_encode,
    rpc_request::RpcError,
};
use solana_sdk::{
    instruction::Instruction, signer::Signer, sysvar::rent, transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgram;

use super::Client;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    derived_addresses,
    error::Error,
    models::*,
};

impl<T> Client<T>
where
    T: Signer + Send + Sync,
{
    pub async fn create_storage_account<'a, 'b>(
        &'a self,
        name: &'b str,
        size: Byte,
    ) -> ShadowDriveResult<CreateStorageAccountResponse<'_>> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let rpc_client = &self.rpc_client;

        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);

        // If userInfo hasn't been initialized, default to 0 for account seed
        let user_info_acct = rpc_client.get_account(&user_info);

        let mut account_seed: u32 = 0;
        match user_info_acct {
            Ok(user_info_acct) => {
                let user_info = UserInfo::try_deserialize(&mut user_info_acct.data.as_slice())
                    .map_err(Error::AnchorError)?;
                account_seed = user_info.account_counter;
            }
            Err(ClientError {
                kind: ClientErrorKind::RpcError(RpcError::ForUser(_)),
                ..
            }) => {
                // this is what rpc_client.get_account() returns if the account doesn't exist
                // assume 0 seed
            }
            Err(err) => {
                //a different rpc error occurred
                return Err(Error::from(err));
            }
        }

        let storage_requested: u64 = size
            .get_bytes()
            .try_into()
            .map_err(|_| Error::InvalidStorage)?;

        let (storage_account, _) = derived_addresses::storage_account(&wallet_pubkey, account_seed);

        let (stake_account, _) = derived_addresses::stake_account(&storage_account);

        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::InitializeStorageAccount {
            storage_config: *STORAGE_CONFIG_PDA,
            user_info,
            storage_account,
            stake_account,
            token_mint: TOKEN_MINT,
            owner_1: wallet_pubkey,
            uploader: UPLOADER,
            owner_1_token_account: owner_ata,
            system_program: system_program::ID,
            token_program: TokenProgram,
            rent: rent::ID,
        };

        let args = InitializeAccount {
            identifier: name.to_string(),
            storage: storage_requested,
            owner_2: None,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));

        txn.try_partial_sign(&[wallet], rpc_client.get_latest_blockhash()?)?;

        let txn_encoded = serialize_and_encode(&txn, UiTransactionEncoding::Base64)?;

        let body = serde_json::to_string(&json!({ "transaction": txn_encoded })).unwrap();

        let response = self
            .http_client
            .post(format!("{}/storage-account", SHDW_DRIVE_ENDPOINT))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ShadowDriveServerError {
                status: response.status().as_u16(),
                message: response.json::<Value>().await?,
            });
        }

        let response = response.json::<CreateStorageAccountResponse<'_>>().await?;

        Ok(response)
    }
}
