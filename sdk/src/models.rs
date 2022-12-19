use bytes::Bytes;
use reqwest::multipart::Part;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt};

//re-export structs from Shadow Drive Smart Contract that are used in the SDK
pub use shadow_drive_user_staking::instructions::{
    decrease_storage::UnstakeInfo, initialize_account::UserInfo, store_file::File as FileAccount,
};

pub mod payload;
pub mod storage_acct;

use crate::{constants::FILE_SIZE_LIMIT, error::Error};
use payload::Payload;

pub type ShadowDriveResult<T> = Result<T, Error>;

const BUFFER_SIZE: usize = 4096;

#[derive(Clone, Debug, Deserialize)]
pub struct ShdwDriveResponse {
    pub txid: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageResponse {
    pub message: String,
    pub transaction_signature: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateStorageAccountResponse {
    pub shdw_bucket: Option<String>,
    pub transaction_signature: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeleteFileResponse {
    pub message: String,
    pub error: Option<String>,
}

/// [`ShadowFile`] is the combination of a file name and a [`Payload`].
#[derive(Debug)]
pub struct ShadowFile {
    pub name: String,
    pub data: Payload,
    content_type: Option<String>,
}

impl ShadowFile {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn with_content_type(mut self, content_type: String) -> Self {
        self.content_type = Some(content_type);
        self
    }

    pub fn file<T: AsRef<Path>>(name: String, path: T) -> Self {
        Self {
            name,
            content_type: None,
            data: Payload::File(path.as_ref().to_owned()),
        }
    }

    pub fn bytes<T: Into<Bytes>>(name: String, data: T) -> Self {
        Self {
            name,
            content_type: None,
            data: Payload::Bytes(data.into()),
        }
    }

    pub(crate) async fn sha256(&self) -> ShadowDriveResult<String> {
        let result = match &self.data {
            Payload::File(path) => {
                let mut file = File::open(path).await.map_err(Error::FileSystemError)?;
                let mut buf = [0u8; BUFFER_SIZE];
                let mut hasher = Sha256::new();

                loop {
                    let bytes_read = file.read(&mut buf[..]).await?;

                    if bytes_read != 0 {
                        hasher.update(&buf[..bytes_read]);
                    } else {
                        break;
                    }
                }

                hasher.finalize()
            }
            Payload::Bytes(data) => {
                let mut hasher = Sha256::new();
                hasher.update(&data);
                hasher.finalize()
            }
        };
        Ok(hex::encode(result))
    }

    pub(crate) async fn into_form_part(self) -> ShadowDriveResult<Part> {
        let mut part = match self.data {
            Payload::File(path) => {
                let file = File::open(path).await.map_err(Error::FileSystemError)?;
                let file_meta = file.metadata().await.map_err(Error::FileSystemError)?;

                //make sure that the file is under the size limit
                if file_meta.len() > FILE_SIZE_LIMIT {
                    return Err(Error::FileTooLarge(self.name.clone()));
                }

                Part::stream_with_length(file, file_meta.len()).file_name(self.name)
            }
            Payload::Bytes(data) => {
                //make sure that the file is under the size limit
                if data.len() as u64 > FILE_SIZE_LIMIT {
                    return Err(Error::FileTooLarge(self.name.clone()));
                }

                Part::stream_with_length(Bytes::clone(&data), data.len() as u64)
                    .file_name(self.name)
            }
        };

        if let Some(content_type) = self.content_type {
            part = part.mime_str(&content_type)?;
        };
        Ok(part)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShadowUploadResponse {
    #[serde(default)]
    pub finalized_locations: Vec<String>,
    pub message: String,
    #[serde(default)]
    pub upload_errors: Vec<UploadError>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UploadError {
    pub file: String,
    pub storage_account: String,
    pub error: String,
}

#[allow(dead_code)]
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
    pub owner_account_pubkey: String,
    pub storage_account_pubkey: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ListObjectsResponse {
    pub keys: Vec<String>,
}
