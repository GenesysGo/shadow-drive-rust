use anchor_lang::error::Error as AnchorError;
use reqwest::Error as ReqwestError;
use solana_client::client_error::ClientError;
use solana_sdk::pubkey::ParsePubkeyError;
use solana_sdk::signer::SignerError;
use std::io::Error as IoError;
use tokio::task::JoinError;

#[derive(Debug)]
pub enum Error {
    ShadowDriveServerError {
        status: u16,
        message: serde_json::Value,
    },
    FileTooLarge(String),
    TransactionSerializationFailed(String),
    InvalidJson(serde_json::Error),
    SolanaRpcError(ClientError),
    AccountDeserializeError(IoError),
    InvalidStorage,
    SignerError(SignerError),
    AnchorError(AnchorError),
    ReqwestError(ReqwestError),
    AsyncJoinError(JoinError),
    FileValidationError(Vec<FileError>),
    UserInfoNotCreated,
    FileSystemError(std::io::Error),
    ParsePubkeyError(ParsePubkeyError),
    NotFileOwner,
    StorageAccountIsNotImmutable,
}

#[derive(Debug)]
pub struct FileError {
    pub file: String,
    pub error: String,
}

impl From<JoinError> for Error {
    fn from(join_error: JoinError) -> Self {
        Self::AsyncJoinError(join_error)
    }
}

impl From<ClientError> for Error {
    fn from(client_error: ClientError) -> Self {
        Self::SolanaRpcError(client_error)
    }
}

impl From<SignerError> for Error {
    fn from(signer_error: SignerError) -> Self {
        Self::SignerError(signer_error)
    }
}

impl From<AnchorError> for Error {
    fn from(anchor_error: AnchorError) -> Self {
        Self::AnchorError(anchor_error)
    }
}

impl From<ReqwestError> for Error {
    fn from(signer_error: ReqwestError) -> Self {
        Self::ReqwestError(signer_error)
    }
}

impl From<ParsePubkeyError> for Error {
    fn from(parse_pubkey_error: ParsePubkeyError) -> Self {
        Self::ParsePubkeyError(parse_pubkey_error)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::FileSystemError(e)
    }
}
