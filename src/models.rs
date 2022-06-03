use serde::Deserialize;
use std::borrow::Cow;
use tokio::fs;

use crate::error::Error;

pub type ShadowDriveResult<T> = Result<T, Error>;

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
