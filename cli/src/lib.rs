use std::{io::stdin, str::FromStr};

use anyhow::anyhow;
use byte_unit::Byte;
use chrono::DateTime;
use reqwest::{header::HeaderMap, Response};
use shadow_drive_sdk::{
    constants::SHDW_DRIVE_OBJECT_PREFIX,
    error::{Error, FileError},
    models::ShadowDriveResult,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signature, Signer, SignerError},
};

/// Maximum amount of files to batch into a single [store_files] request.
pub const FILE_UPLOAD_BATCH_SIZE: usize = 5;

/// Clap value parser for base58 string representations of [Pubkey].
pub fn pubkey_arg(pubkey: &str) -> anyhow::Result<Pubkey> {
    Pubkey::from_str(pubkey).map_err(|e| anyhow!("invalid pubkey: {}", e.to_string()))
}

/// To get around using a [Box<dyn Signer>] with [ShadowDriveClient].
pub struct WrappedSigner(Box<dyn Signer>);

impl WrappedSigner {
    pub fn new(signer: Box<dyn Signer>) -> Self {
        Self(signer)
    }
}

impl Signer for WrappedSigner {
    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        Ok(self.0.pubkey())
    }

    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        self.0.try_sign_message(message)
    }

    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
}

/// Further diagnostic printing wherever possible.
pub fn process_shadow_api_response<T>(response: ShadowDriveResult<T>) -> anyhow::Result<T> {
    match response {
        Ok(response) => Ok(response),
        Err(err) => match err {
            Error::ShadowDriveServerError { status, message } => {
                let err = format!(
                    "Shadow Drive Server Error {}: {:#?}",
                    status,
                    message.to_string()
                );
                println!("{}", err);
                Err(anyhow!("{}", err))
            }
            Error::FileSystemError(err) => {
                let err = format!("Filesystem Error: {:#?}", err.to_string());
                println!("{}", err);
                Err(anyhow!("{}", err))
            }
            Error::FileValidationError(errs) => {
                let mut err_vec = vec![];
                for err in errs {
                    let FileError { file, error } = err;
                    let err = format!("File Validation Error for {}: {}", file, error);
                    err_vec.push(err);
                }
                println!("{:#?}", err_vec);
                Err(anyhow!("{:#?}", err_vec))
            }
            e => {
                println!("{:#?}", e);
                Err(anyhow!("{:#?}", e))
            }
        },
    }
}

/// Generate a Shadow Drive file URL from storage account and filename.
pub fn storage_object_url(storage_account: &Pubkey, file: &str) -> String {
    format!(
        "{}/{}/{}",
        SHDW_DRIVE_OBJECT_PREFIX,
        storage_account.to_string(),
        file
    )
}

/// Returns false when "Content-Type" header is not "text/plain".
fn is_text_response(headers: &HeaderMap) -> anyhow::Result<bool> {
    let content_type = headers
        .get("content-type")
        .and_then(|s| Some(s.to_str()))
        .transpose()?;
    Ok(content_type == Some("text/plain"))
}

/// Check with a HEAD that the URL exists and is a "text/plain" file.
/// If so, return the response of a GET request.
pub async fn get_text(url: &String) -> anyhow::Result<Response> {
    let http_client = reqwest::Client::new();
    let head_resp = http_client.head(url).send().await?;
    if !is_text_response(head_resp.headers())? {
        return Err(anyhow!("Not a text file at url {}", url));
    }
    Ok(http_client.get(url).send().await?)
}

#[derive(Debug)]
pub struct FileMetadata {
    pub timestamp: i64,
    pub content_type: String,
    pub last_modified: i64,
    pub etag: String,
    pub storage_account: String,
    pub storage_owner: String,
}

impl FileMetadata {
    pub fn from_headers(h: &HeaderMap) -> anyhow::Result<Self> {
        let getter = |key| {
            Ok::<_, anyhow::Error>(
                h.get(key)
                    .ok_or(anyhow!("Missing file metadata header: {}", key))?
                    .to_str()?
                    .to_string(),
            )
        };
        let parse_timestamp = |key| {
            let timestamp = getter(key)?;
            let timestamp = DateTime::parse_from_rfc2822(&timestamp)?;
            Ok::<_, anyhow::Error>(timestamp.timestamp())
        };
        let timestamp = parse_timestamp("date")?;
        let last_modified = parse_timestamp("last-modified")?;
        Ok(Self {
            timestamp,
            content_type: getter("content-type")?,
            last_modified,
            etag: getter("etag")?,
            storage_account: getter("x-amz-meta-owner-account-pubkey")?,
            storage_owner: getter("x-amz-meta-storage-account-pubkey")?,
        })
    }
}

/// Pulls "last-modified" from [HeaderMap], unaltered.
pub fn last_modified(headers: &HeaderMap) -> anyhow::Result<String> {
    Ok(headers
        .get("last-modified")
        .ok_or(anyhow!("'last modified' header not found"))?
        .to_str()?
        .to_string())
}

/// Convert a file size string to [Byte] object with the denoted size.
pub fn parse_filesize(size: &str) -> anyhow::Result<Byte> {
    Byte::from_str(size).map_err(|e| {
        anyhow!(
            "invalid filesize, \
        expected a number followed by KB, MB, GB:\n{}",
            e.to_string()
        )
    })
}

/// Confirm from the user that they definitely want some irreversible
/// operation to occur.
pub fn wait_for_user_confirmation(skip: bool) -> anyhow::Result<()> {
    if skip {
        return Ok(());
    }
    println!("Press ENTER to continue, or CTRL+C to abort");
    let mut proceed = String::new();
    stdin().read_line(&mut proceed)?;
    Ok(())
}
