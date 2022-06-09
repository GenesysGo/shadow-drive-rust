use bytes::Bytes;
use cryptohelpers::sha256;
use reqwest::multipart::Part;
use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};

use crate::{
    constants::{FILE_SIZE_LIMIT, SHDW_DRIVE_OBJECT_PREFIX},
    error::{Error, FileError},
};

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
    pub name: String,
    pub file: fs::File,
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

#[derive(Debug)]
pub enum ShadowFile {
    File { name: String, path: PathBuf },
    Buf { name: String, data: Bytes },
}

impl ShadowFile {
    pub fn name(&self) -> &str {
        match self {
            ShadowFile::File { name, .. } => name.as_str(),
            ShadowFile::Buf { name, .. } => name.as_str(),
        }
    }
    pub fn file<T: AsRef<Path>>(name: String, path: T) -> Self {
        Self::File {
            name,
            path: path.as_ref().to_owned(),
        }
    }

    pub fn bytes<T: Into<Bytes>>(name: String, data: T) -> Self {
        Self::Buf {
            name,
            data: data.into(),
        }
    }

    pub async fn prepare_upload(
        self,
        storage_account_key: &Pubkey,
    ) -> Result<UploadingData, Vec<FileError>> {
        match self {
            Self::File { name, path } => prepare_file_upload(storage_account_key, name, path).await,
            Self::Buf { name, data } => prepare_buf_upload(storage_account_key, name, data).await,
        }
    }

    pub async fn to_form_part(&self, size: u64) -> ShadowDriveResult<Part> {
        match self {
            Self::File { name, path } => {
                let file = File::open(path).await.map_err(Error::FileSystemError)?;
                Ok(Part::stream_with_length(file, size).file_name(name.clone()))
            }
            Self::Buf { name, data } => {
                Ok(Part::stream_with_length(Bytes::clone(&data), size).file_name(name.clone()))
            }
        }
    }
}

pub(crate) async fn prepare_buf_upload(
    storage_account_key: &Pubkey,
    file_name: String,
    data: Bytes,
) -> Result<UploadingData, Vec<FileError>> {
    let mut errors = Vec::new();
    let file_size = data.len() as u64;
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
    let sha256_hash = match sha256::compute(&mut data.as_ref()).await {
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
        file: ShadowFile::Buf {
            name: file_name,
            data,
        },
    })
}

pub(crate) async fn prepare_file_upload(
    storage_account_key: &Pubkey,
    file_name: String,
    path: PathBuf,
) -> Result<UploadingData, Vec<FileError>> {
    let mut file = File::open(&path).await.map_err(|err| {
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
        file: ShadowFile::File {
            name: file_name,
            path,
        },
    })
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
