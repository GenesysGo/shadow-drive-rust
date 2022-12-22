
use anyhow::anyhow;
use byte_unit::Byte;
use reqwest::header::HeaderMap;
use reqwest::Response;
use shadow_drive_sdk::error::{Error, FileError};
use shadow_drive_sdk::models::ShadowDriveResult;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Signature, Signer, SignerError};
use std::io::stdin;
use std::path::PathBuf;
use std::str::FromStr;

/// Shadow Drive Files are hosted at this domain.
pub const GENESYSGO_DRIVE: &str = "https://shdw-drive.genesysgo.net";

/// Maximum amount of files to batch into a single [store_files] request.
pub const FILE_UPLOAD_BATCH_SIZE: usize = 5;

/// Clap value parser for base58 string representations of [Pubkey].
pub fn pubkey_arg(pubkey: &str) -> anyhow::Result<Pubkey> {
    Pubkey::from_str(pubkey).map_err(
        |e| anyhow!("invalid pubkey: {}", e.to_string())
    )
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

    fn sign_message(&self, message: &[u8]) -> Signature {
        self.try_sign_message(message).unwrap()
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
pub fn drive_url(storage_account: &Pubkey, file: &str) -> String {
    format!(
        "{}/{}/{}",
        GENESYSGO_DRIVE,
        storage_account.to_string(),
        file
    )
}

/// Pull the basename off of a filepath. Useful for automatically
/// generating filenames during upload.
pub fn acquire_basename(path: &str) -> String {
    let path = PathBuf::from_str(path).expect(&format!("not a valid path {}", path));
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap()
        .to_string()
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
