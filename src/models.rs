use serde::Deserialize;
use std::borrow::Cow;
use tokio::fs;

use crate::error::Error;

pub type ShadowDriveResult<T> = Result<T, Error>;

#[derive(Clone, Debug, Deserialize)]
pub struct ShdwDriveResponse<'a> {
    pub txid: Cow<'a, str>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateStorageAccountResponse<'a> {
    pub shdw_bucket: Option<Cow<'a, str>>,
    pub transaction_signature: Cow<'a, str>,
}

/// A ShdwFile is the pairing of a filename w/ bytes to be uploaded
#[derive(Debug)]
pub struct ShdwFile {
    pub name: Option<String>,
    pub file: fs::File,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShadowUploadResponse<'a> {
    pub finalized_location: Cow<'a, str>,
    pub transaction_signature: Cow<'a, str>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FileDataResponse {
    pub file_data: FileData
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FileData {
    pub file_account_pubkey: String,
    pub owner_account_pubkey: String,
    pub storage_account_pubkey: String,
}
