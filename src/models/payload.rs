use crate::{
    constants::{FILE_SIZE_LIMIT, SHDW_DRIVE_OBJECT_PREFIX},
    error::FileError,
};
use async_trait::async_trait;
use bytes::Bytes;
use cryptohelpers::sha256;
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use tokio::fs::File;

use super::{ShadowFile, UploadingData};

/// [`Payload`] is an enum containing the types that the
/// SDK can upload to ShadowDrive. Each variant is expected to implement [`PayloadExt`] so the SDK
/// can derive required upload metadata.
#[derive(Debug)]
pub enum Payload {
    File(PathBuf),
    Bytes(Bytes),
}

impl Payload {
    pub async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
        file_name: String,
    ) -> Result<UploadingData, Vec<FileError>> {
        match self {
            Self::File(path) => path.prepare_upload(storage_account_key, file_name).await,
            Self::Bytes(data) => data.prepare_upload(storage_account_key, file_name).await,
        }
    }
}

/// [`PayloadExt`] is used to implement new data sources that can be used in the [`Payload`] enum.
#[async_trait]
pub trait PayloadExt {
    /// prepare_upload receives a storage account [`Pubkey`] and a file name. These details are used
    /// in combination with the underlying type to derive:
    /// * data size in bytes
    /// * sha256 of the data
    /// * the url that the file will be accessible at upon successsful upload
    async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
        file_name: String,
    ) -> Result<UploadingData, Vec<FileError>>;
}

#[async_trait]
impl PayloadExt for PathBuf {
    async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
        file_name: String,
    ) -> Result<UploadingData, Vec<FileError>> {
        let mut file = File::open(&self).await.map_err(|err| {
            vec![FileError {
                file: file_name.clone(),
                error: format!("error opening file: {:?}", err),
            }]
        })?;

        let file_meta = file.metadata().await.map_err(|err| {
            vec![FileError {
                file: file_name.clone(),
                error: format!("error opening file metadata: {:?}", err),
            }]
        })?;

        let mut errors = Vec::new();
        let file_size = file_meta.len();
        if file_size > FILE_SIZE_LIMIT {
            errors.push(FileError {
                file: file_name.clone(),
                error: String::from("Exceed the 1GB limit."),
            });
        }

        //store any info about file bytes before moving into form
        let sha256_hash = match sha256::compute(&mut file).await {
            Ok(hash) => hash,
            Err(err) => {
                errors.push(FileError {
                    file: file_name.clone(),
                    error: format!("error hashing file: {:?}", err),
                });
                return Err(errors);
            }
        };

        if file_name.as_bytes().len() > 32 {
            errors.push(FileError {
                file: file_name.clone(),
                error: String::from("Exceed the 1GB limit."),
            });
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        //this may need to be url encoded
        let url = format!(
            "{}/{}/{}",
            SHDW_DRIVE_OBJECT_PREFIX,
            storage_account_key.to_string(),
            &file_name
        );

        Ok(UploadingData {
            size: file_size,
            sha256_hash,
            url,
            file: ShadowFile {
                name: file_name.clone(),
                data: Payload::File(self),
            },
        })
    }
}

#[async_trait]
impl PayloadExt for Bytes {
    async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
        file_name: String,
    ) -> Result<UploadingData, Vec<FileError>> {
        let mut errors = Vec::new();
        let file_size = self.len() as u64;
        if file_size > FILE_SIZE_LIMIT {
            errors.push(FileError {
                file: file_name.clone(),
                error: String::from("Exceed the 1GB limit."),
            });
        }

        if file_name.as_bytes().len() > 32 {
            errors.push(FileError {
                file: file_name.clone(),
                error: String::from("Exceed the 1GB limit."),
            });
        }

        //store any info about file bytes before moving into form
        let sha256_hash = match sha256::compute(&mut self.as_ref()).await {
            Ok(hash) => hash,
            Err(err) => {
                errors.push(FileError {
                    file: file_name.clone(),
                    error: format!("error hashing file: {:?}", err),
                });
                return Err(errors);
            }
        };

        if errors.len() > 0 {
            return Err(errors);
        }

        //this may need to be url encoded
        let url = format!(
            "{}/{}/{}",
            SHDW_DRIVE_OBJECT_PREFIX,
            storage_account_key.to_string(),
            &file_name
        );

        Ok(UploadingData {
            size: file_size,
            sha256_hash,
            url,
            file: ShadowFile {
                name: file_name,
                data: Payload::Bytes(self),
            },
        })
    }
}
