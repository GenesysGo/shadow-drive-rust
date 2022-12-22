use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCodes {
    // New Global Error Codes
    #[msg("Not enough storage available on this Storage Account")]
    NotEnoughStorage,
    #[msg("The length of the file name exceeds the limit of 32 bytes")]
    FileNameLengthExceedsLimit,
    #[msg("Invalid sha256 hash")]
    InvalidSha256Hash,
    #[msg("User at some point had a bad csam scan")]
    HasHadBadCsam,
    #[msg("Storage account is marked as immutable")]
    StorageAccountMarkedImmutable,
    #[msg("User has not waited enough time to claim stake")]
    ClaimingStakeTooSoon,
    #[msg("The storage account needs to be marked as mutable to update last fee collection epoch")]
    SolanaStorageAccountNotMutable,
    #[msg("Attempting to decrease storage by more than is available")]
    RemovingTooMuchStorage,
    #[msg("u128 -> u64 cast failed")]
    UnsignedIntegerCastFailed,
    #[msg("This storage account still has some file accounts associated with it that have not been deleted")]
    NonzeroRemainingFileAccounts,
    #[msg("This account is still within deletion grace period")]
    AccountStillInGracePeriod,
    #[msg("This account is not marked to be deleted")]
    AccountNotMarkedToBeDeleted,
    #[msg("This file is still within deletion grace period")]
    FileStillInGracePeriod,
    #[msg("This file is not marked to be deleted")]
    FileNotMarkedToBeDeleted,
    #[msg("File has been marked as immutable and cannot be edited")]
    FileMarkedImmutable,
    #[msg("User requested an increase of zero bytes")]
    NoStorageIncrease,
    #[msg("Requested a storage account with storage over the limit")]
    ExceededStorageLimit,
    #[msg("User does not have enough funds to store requested number of bytes.")]
    InsufficientFunds,
    #[msg("There is not available storage on Shadow Drive. Good job!")]
    NotEnoughStorageOnShadowDrive,
    #[msg("Requested a storage account with storage under the limit")]
    AccountTooSmall,
    #[msg("User did not agree to terms of service")]
    DidNotAgreeToToS,
    #[msg("Invalid token transfers. Stake account nonempty.")]
    InvalidTokenTransferAmounts,
    #[msg("Failed to close spl token account")]
    FailedToCloseAccount,
    #[msg("Failed to transfer to emissions wallet")]
    FailedToTransferToEmissionsWallet,
    #[msg("Failed to transfer to emissions wallet from user")]
    FailedToTransferToEmissionsWalletFromUser,
    #[msg("Failed to return user funds")]
    FailedToReturnUserFunds,
    #[msg("Turning on fees and passing in None for storage cost per epoch")]
    NeedSomeFees,
    #[msg("Turning on fees and passing in None for crank bps")]
    NeedSomeCrankBps,
    #[msg("This account is already marked to be deleted")]
    AlreadyMarkedForDeletion,
    #[msg("User has an empty stake account and must refresh stake account before unmarking account for deletion")]
    EmptyStakeAccount,
    #[msg("New identifier exceeds maximum length of 64 bytes")]
    IdentifierExceededMaxLength,
    #[msg("Only admin1 can change admins")]
    OnlyAdmin1CanChangeAdmins,
    #[msg{("As part of on-chain storage optimizations, only one owner is allowed in Shadow Drive v1.5")}]
    OnlyOneOwnerAllowedInV1_5,
}
