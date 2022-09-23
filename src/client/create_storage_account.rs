use anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas};
use byte_unit::Byte;
use serde_json::{json, Value};
use shadow_drive_user_staking::instruction::InitializeAccount;
use shadow_drive_user_staking::instructions::initialize_account::UserInfo;
use shadow_drive_user_staking::{accounts as shdw_drive_accounts, instruction::InitializeAccount2};
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::RpcError,
};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signer::Signer, sysvar::rent,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::ID as TokenProgram;

use super::ShadowDriveClient;
use crate::{
    constants::{PROGRAM_ADDRESS, SHDW_DRIVE_ENDPOINT, STORAGE_CONFIG_PDA, TOKEN_MINT, UPLOADER},
    derived_addresses,
    error::Error,
    models::*,
    serialize_and_encode,
};

pub enum StorageAccountVersion {
    V1 { owner_2: Option<Pubkey> },
    V2,
}

impl StorageAccountVersion {
    pub fn v1() -> Self {
        Self::V1 { owner_2: None }
    }

    pub fn v1_with_owner_2(owner_2: Pubkey) -> Self {
        Self::V1 {
            owner_2: Some(owner_2),
        }
    }

    pub fn v2() -> Self {
        Self::V2
    }
}

impl<T> ShadowDriveClient<T>
where
    T: Signer,
{
    /// Creates a [`StorageAccount`](crate::models::StorageAccount) on the Shadow Drive.
    /// [`StorageAccount`]'s can hold multiple files, and are paid for using the SHDW token.
    /// * `name` - The name of the [`StorageAccount`](crate::models::StorageAccount). Does not need to be unique.
    /// * `size` - The amount of storage the [`StorageAccount`](crate::models::StorageAccount) should be initialized with.
    /// When specifying size, only KB, MB, and GB storage units are currently supported.
    pub async fn create_storage_account(
        &self,
        name: &str,
        size: Byte,
        version: StorageAccountVersion,
    ) -> ShadowDriveResult<CreateStorageAccountResponse> {
        let wallet = &self.wallet;
        let wallet_pubkey = wallet.pubkey();

        let rpc_client = &self.rpc_client;

        let (user_info, _) = derived_addresses::user_info(&wallet_pubkey);

        // If userInfo hasn't been initialized, default to 0 for account seed
        let user_info_acct = rpc_client.get_account(&user_info).await;

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

        let txn_encoded = match version {
            StorageAccountVersion::V1 { owner_2 } => {
                self.create_v1(name, account_seed, user_info, storage_requested, owner_2)
                    .await?
            }
            StorageAccountVersion::V2 => {
                self.create_v2(name, account_seed, user_info, storage_requested)
                    .await?
            }
        };

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

        let response = response.json::<CreateStorageAccountResponse>().await?;

        Ok(response)
    }

    async fn create_v1(
        &self,
        name: &str,
        account_seed: u32,
        user_info: Pubkey,
        storage_requested: u64,
        owner_2: Option<Pubkey>,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();

        let (storage_account, _) = derived_addresses::storage_account(&wallet_pubkey, account_seed);

        let (stake_account, _) = derived_addresses::stake_account(&storage_account);

        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::InitializeStorageAccountV1 {
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
            owner_2,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));

        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }

    async fn create_v2(
        &self,
        name: &str,
        account_seed: u32,
        user_info: Pubkey,
        storage_requested: u64,
    ) -> ShadowDriveResult<String> {
        let wallet_pubkey = self.wallet.pubkey();

        let (storage_account, _) = derived_addresses::storage_account(&wallet_pubkey, account_seed);

        let (stake_account, _) = derived_addresses::stake_account(&storage_account);

        let owner_ata = get_associated_token_address(&wallet_pubkey, &TOKEN_MINT);

        let accounts = shdw_drive_accounts::InitializeStorageAccountV2 {
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

        let args = InitializeAccount2 {
            identifier: name.to_string(),
            storage: storage_requested,
        };

        let instruction = Instruction {
            program_id: PROGRAM_ADDRESS,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let mut txn = Transaction::new_with_payer(&[instruction], Some(&wallet_pubkey));

        txn.try_partial_sign(
            &[&self.wallet],
            self.rpc_client.get_latest_blockhash().await?,
        )?;

        let txn_encoded = serialize_and_encode(&txn)?;

        Ok(txn_encoded)
    }
}
