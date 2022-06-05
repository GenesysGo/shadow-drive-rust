use serde::Deserialize;
use tokio::fs;

use crate::error::Error;

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

/// A ShdwFile is the pairing of a filename w/ bytes to be uploaded
#[derive(Debug)]
pub struct ShdwFile {
    pub name: Option<String>,
    pub file: fs::File,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShadowUploadResponse {
    pub finalized_location: String,
    pub transaction_signature: String,
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
