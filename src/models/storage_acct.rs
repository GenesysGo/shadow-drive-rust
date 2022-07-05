use anchor_lang::{AccountDeserialize, Discriminator};
use shadow_drive_user_staking::instructions::initialize_account::{
    StorageAccount, StorageAccountV1, StorageAccountV2,
};

type Storage = u64;
type Epoch = u32;

pub enum StorageAcct {
    V1(StorageAccountV1),
    V2(StorageAccountV2),
}

impl StorageAcct {
    pub fn deserialize(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
        match &buf[..8] {
            discriminator if discriminator == StorageAccountV2::discriminator() => {
                <StorageAccountV2 as AccountDeserialize>::try_deserialize_unchecked(buf)
                    .map(Self::V2)
            }
            discriminator if discriminator == StorageAccountV1::discriminator() => {
                StorageAccountV1::try_deserialize_unchecked(buf).map(Self::V1)
            }
            _ => Err(anchor_lang::error::ErrorCode::AccountDiscriminatorMismatch.into()),
        }
    }
}

macro_rules! storage_acct_getter {
    ($method: ident, $return: ident) => {
        fn $method(&self) -> $return {
            match self {
                Self::V1(v1) => v1.$method(),
                Self::V2(v2) => v2.$method(),
            }
        }
    };
}

macro_rules! storage_acct_setter {
    ($method: ident) => {
        fn $method(&mut self) {
            match self {
                Self::V1(v1) => v1.$method(),
                Self::V2(v2) => v2.$method(),
            }
        }
    };
}

impl StorageAccount for StorageAcct {
    storage_acct_getter!(check_immutable, bool);
    storage_acct_getter!(check_delete_flag, bool);
    storage_acct_getter!(get_identifier, String);
    storage_acct_getter!(get_storage, Storage);
    storage_acct_getter!(get_last_fee_epoch, Epoch);

    storage_acct_setter!(mark_to_delete);
    storage_acct_setter!(update_last_fee_epoch);
}
