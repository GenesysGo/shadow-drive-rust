use bytes::Bytes;
use cryptohelpers::sha256;
use reqwest::multipart::Part;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::path::Path;
use tokio::fs::File;

pub mod payload;

use crate::error::{Error, FileError};
use payload::Payload;

pub type ShadowDriveResult<T> = Result<T, Error>;

#[derive(Clone, Debug, Deserialize)]
pub struct ShdwDriveResponse {
    pub txid: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateStorageAccountResponse {
    pub shdw_bucket: Option<String>,
    pub transaction_signature: String,
}

/// UploadingData is a collection of info required for uploading a file
/// to Shadow Drive. Fields are generally derived from a given [`ShdwFile`] during the upload process.
#[derive(Debug)]
pub struct UploadingData {
    pub size: u64,
    pub sha256_hash: sha256::Sha256Hash,
    pub url: String,
    pub file: ShadowFile,
}

impl UploadingData {
    pub async fn to_form_part(&self) -> ShadowDriveResult<Part> {
        match &self.file.data {
            Payload::File(path) => {
                let file = File::open(path).await.map_err(Error::FileSystemError)?;
                Ok(Part::stream_with_length(file, self.size).file_name(self.file.name.clone()))
            }
            Payload::Bytes(data) => Ok(Part::stream_with_length(Bytes::clone(&data), self.size)
                .file_name(self.file.name.clone())),
        }
    }
}

/// [`ShadowFile`] is the combination of a file name and a [`Payload`].
#[derive(Debug)]
pub struct ShadowFile {
    pub name: String,
    pub data: Payload,
}

impl ShadowFile {
    pub fn file<T: AsRef<Path>>(name: String, path: T) -> Self {
        Self {
            name,
            data: Payload::File(path.as_ref().to_owned()),
        }
    }

    pub fn bytes<T: Into<Bytes>>(name: String, data: T) -> Self {
        Self {
            name,
            data: Payload::Bytes(data.into()),
        }
    }

    pub async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
    ) -> Result<UploadingData, Vec<FileError>> {
        Payload::prepare_upload(self.data, storage_account_key, self.name).await
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShadowUploadResponse {
    pub finalized_location: String,
    pub transaction_signature: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ShdwDriveBatchServerResponse {
    pub _finalized_locations: Option<Vec<String>>,
    pub transaction_signature: String,
}

#[derive(Clone, Debug, Deserialize)]
pub enum BatchUploadStatus {
    Uploaded,
    AlreadyExists,
    Error(String),
}
#[derive(Clone, Debug, Deserialize)]
pub struct ShadowBatchUploadResponse {
    pub file_name: String,
    pub status: BatchUploadStatus,
    pub location: Option<String>,
    pub transaction_signature: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FileDataResponse {
    pub file_data: FileData,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FileData {
    pub file_account_pubkey: String,
    pub owner_account_pubkey: String,
    pub storage_account_pubkey: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListObjectsResponse {
    pub keys: Vec<String>,
}
