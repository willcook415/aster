use super::JournalError;
use crate::{CommandBatchV2, SequencedEngine};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};

pub(super) const HEADER: &[u8; 16] = b"ASTERJNL\x01\0\0\0\x02\0\x01\0";
const COMMIT: &[u8; 8] = b"ASTRCMIT";
const MAX_FRAME: usize = 16 * 1024 * 1024;

pub(super) struct Scan {
    pub engine: SequencedEngine,
    pub committed_bytes: u64,
    pub tail_bytes: u64,
    pub digest: [u8; 32],
}

fn digest(previous: &[u8; 32], header: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(previous);
    hash.update(header);
    hash.update(payload);
    hash.finalize().into()
}

pub(super) fn encode(
    batch: &CommandBatchV2,
    previous: &[u8; 32],
) -> Result<(Vec<u8>, [u8; 32]), JournalError> {
    let payload = serde_json::to_vec(batch)?;
    if payload.len() > MAX_FRAME {
        return Err(JournalError::FrameTooLarge);
    }
    let len = payload.len() as u32;
    let mut header = Vec::from(len.to_le_bytes());
    header.extend_from_slice(&(!len).to_le_bytes());
    let checksum = digest(previous, &header, &payload);
    let mut bytes = header;
    bytes.extend_from_slice(&payload);
    bytes.extend_from_slice(&checksum);
    bytes.extend_from_slice(COMMIT);
    Ok((bytes, checksum))
}

pub(super) trait SyncWriter: Write {
    fn sync_commit(&self) -> std::io::Result<()>;
}
impl SyncWriter for File {
    fn sync_commit(&self) -> std::io::Result<()> {
        self.sync_all()
    }
}

/// Success is returned only after both the final marker and sync have succeeded.
pub(super) fn write_frame(writer: &mut impl SyncWriter, bytes: &[u8]) -> std::io::Result<()> {
    writer.write_all(bytes)?;
    writer.sync_commit()
}

pub(super) fn scan(file: &mut File) -> Result<Scan, JournalError> {
    file.seek(SeekFrom::Start(0))?;
    let length = file.metadata()?.len();
    if length < HEADER.len() as u64 {
        return Err(JournalError::InvalidHeader);
    }
    let mut header = [0; 16];
    file.read_exact(&mut header)?;
    if &header != HEADER {
        return Err(JournalError::InvalidHeader);
    }
    let mut state = Scan {
        engine: SequencedEngine::new(),
        committed_bytes: 16,
        tail_bytes: 0,
        digest: [0; 32],
    };
    while state.committed_bytes < length {
        let offset = state.committed_bytes;
        let remaining = length - offset;
        if remaining < 8 {
            break;
        }
        let mut sizes = [0; 8];
        file.read_exact(&mut sizes)?;
        let len = u32::from_le_bytes([sizes[0], sizes[1], sizes[2], sizes[3]]);
        let complement = u32::from_le_bytes([sizes[4], sizes[5], sizes[6], sizes[7]]);
        let corrupt = |reason: &str| JournalError::Corrupt {
            offset,
            reason: reason.to_owned(),
        };
        if len != !complement || len == 0 || len as usize > MAX_FRAME {
            return Err(corrupt("invalid frame length"));
        }
        let total = 8 + u64::from(len) + 32 + 8;
        if remaining < total {
            break;
        }
        let mut payload = vec![0; len as usize];
        file.read_exact(&mut payload)?;
        let mut saved_digest = [0; 32];
        file.read_exact(&mut saved_digest)?;
        let mut marker = [0; 8];
        file.read_exact(&mut marker)?;
        if &marker != COMMIT {
            return Err(corrupt("invalid commit marker"));
        }
        let checksum = digest(&state.digest, &sizes, &payload);
        if checksum != saved_digest {
            return Err(corrupt("checksum or record-chain mismatch"));
        }
        let batch: CommandBatchV2 =
            serde_json::from_slice(&payload).map_err(|e| JournalError::Corrupt {
                offset,
                reason: e.to_string(),
            })?;
        state
            .engine
            .verify_batch(&batch)
            .map_err(|e| JournalError::Corrupt {
                offset,
                reason: format!("replay verification failed: {e}"),
            })?;
        state.digest = checksum;
        state.committed_bytes += total;
    }
    state.tail_bytes = length - state.committed_bytes;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Faulty {
        bytes: Vec<u8>,
        limit: usize,
        sync_fail: bool,
    }
    impl Write for Faulty {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len() >= self.limit {
                return Err(std::io::Error::other("injected write failure"));
            }
            let count = bytes.len().min(self.limit - self.bytes.len());
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl SyncWriter for Faulty {
        fn sync_commit(&self) -> std::io::Result<()> {
            if self.sync_fail {
                Err(std::io::Error::other("injected sync failure"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn no_acknowledgement_on_partial_write_or_sync_failure() {
        let bytes = b"complete framed record";
        for limit in 0..bytes.len() {
            let mut sink = Faulty {
                bytes: vec![],
                limit,
                sync_fail: false,
            };
            assert!(write_frame(&mut sink, bytes).is_err());
        }
        let mut sink = Faulty {
            bytes: vec![],
            limit: usize::MAX,
            sync_fail: true,
        };
        assert!(write_frame(&mut sink, bytes).is_err());
        assert_eq!(sink.bytes, bytes);
    }
}
