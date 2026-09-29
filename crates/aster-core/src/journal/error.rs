use crate::AsterError;
use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum JournalError {
    Io(io::Error),
    Locked,
    InvalidHeader,
    Corrupt { offset: u64, reason: String },
    Domain(AsterError),
    Serialize(serde_json::Error),
    FrameTooLarge,
    Poisoned,
}
impl fmt::Display for JournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "journal I/O: {e}"),
            Self::Locked => f.write_str("journal is locked by another reader or writer"),
            Self::InvalidHeader => f.write_str("invalid or unsupported journal header"),
            Self::Corrupt { offset, reason } => {
                write!(f, "corrupt journal at byte {offset}: {reason}")
            }
            Self::Domain(e) => write!(f, "journal command failed: {e}"),
            Self::Serialize(e) => write!(f, "journal serialization failed: {e}"),
            Self::FrameTooLarge => f.write_str("journal batch exceeds the 16 MiB frame limit"),
            Self::Poisoned => {
                f.write_str("journal write outcome is uncertain; close and recover before retrying")
            }
        }
    }
}
impl Error for JournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Domain(e) => Some(e),
            Self::Serialize(e) => Some(e),
            _ => None,
        }
    }
}
impl From<io::Error> for JournalError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for JournalError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}
impl From<AsterError> for JournalError {
    fn from(e: AsterError) -> Self {
        Self::Domain(e)
    }
}
