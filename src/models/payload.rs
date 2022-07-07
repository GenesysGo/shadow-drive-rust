use bytes::Bytes;
use std::path::PathBuf;

/// [`Payload`] is an enum containing the types that the
/// SDK can upload to ShadowDrive. Each variant is expected to implement [`PayloadExt`] so the SDK
/// can derive required upload metadata.
#[derive(Debug)]
pub enum Payload {
    File(PathBuf),
    Bytes(Bytes),
}
