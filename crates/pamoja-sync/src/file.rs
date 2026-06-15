//! A crash-safe on-disk store-and-forward queue.

use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use pamoja_core::{Error, Result, Store};

/// The filename extension for a stored record.
const RECORD_EXTENSION: &str = "rec";

/// A durable first-in first-out queue backed by one file per record.
///
/// Each [`append`](Store::append) writes a record to its own sequence-numbered
/// file, flushes it to disk, and atomically renames it into place, so a power
/// loss mid-write leaves the queue consistent: a partially written record is
/// never visible. [`pop`](Store::pop) reads and deletes the oldest record. The
/// directory itself is the durable state, so a store reopened after a crash
/// resumes with every record that was fully written.
///
/// Delivery is at-least-once: if the process stops between reading a record and
/// deleting it, the next [`open`](FileStore::open) returns that record again, so
/// consumers must tolerate the occasional redelivery.
///
/// # Examples
///
/// ```no_run
/// use pamoja_core::Store;
/// use pamoja_sync::FileStore;
///
/// # async fn run() -> pamoja_core::Result<()> {
/// let mut store = FileStore::open("/var/lib/pamoja/outbox")?;
/// store.append(b"reading").await?;
/// if let Some(record) = store.pop().await? {
///     // forward `record` over a transport, then it is gone from the queue
///     let _ = record;
/// }
/// # Ok(())
/// # }
/// ```
pub struct FileStore {
    dir: PathBuf,
    pending: VecDeque<u64>,
    next: u64,
}

impl FileStore {
    /// Opens a store rooted at `dir`, creating the directory if needed.
    ///
    /// Records left in the directory by a previous run are adopted in sequence
    /// order, so the queue resumes where it left off.
    ///
    /// # Arguments
    ///
    /// * `dir` - the directory that holds the queue's record files.
    ///
    /// # Returns
    ///
    /// A store ready to append and drain records.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`](pamoja_core::Error::Io) if the directory cannot be
    /// created or scanned.
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir).map_err(io)?;

        let mut sequences = Vec::new();
        for entry in fs::read_dir(&dir).map_err(io)? {
            let path = entry.map_err(io)?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some(RECORD_EXTENSION) {
                continue;
            }
            if let Some(sequence) = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse::<u64>().ok())
            {
                sequences.push(sequence);
            }
        }
        sequences.sort_unstable();
        let next = sequences.last().map_or(0, |last| last + 1);

        Ok(Self {
            dir,
            pending: sequences.into(),
            next,
        })
    }

    /// Returns the path of the record file for a sequence number.
    fn record_path(&self, sequence: u64) -> PathBuf {
        self.dir.join(format!("{sequence:020}.{RECORD_EXTENSION}"))
    }
}

impl Store for FileStore {
    async fn append(&mut self, record: &[u8]) -> Result<()> {
        let sequence = self.next;
        let final_path = self.record_path(sequence);
        let temp_path = final_path.with_extension(format!("{RECORD_EXTENSION}.tmp"));

        let mut file = fs::File::create(&temp_path).map_err(io)?;
        file.write_all(record).map_err(io)?;
        file.sync_all().map_err(io)?;
        drop(file);
        fs::rename(&temp_path, &final_path).map_err(io)?;

        self.next += 1;
        self.pending.push_back(sequence);
        Ok(())
    }

    async fn peek(&self) -> Result<Option<Vec<u8>>> {
        let Some(&sequence) = self.pending.front() else {
            return Ok(None);
        };
        let record = fs::read(self.record_path(sequence)).map_err(io)?;
        Ok(Some(record))
    }

    async fn pop(&mut self) -> Result<Option<Vec<u8>>> {
        let Some(sequence) = self.pending.pop_front() else {
            return Ok(None);
        };
        let path = self.record_path(sequence);
        let record = fs::read(&path).map_err(io)?;
        fs::remove_file(&path).map_err(io)?;
        Ok(Some(record))
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.pending.len())
    }
}

/// Maps a filesystem error onto the shared I/O error.
fn io(error: std::io::Error) -> Error {
    Error::Io(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn drains_in_first_in_first_out_order() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = FileStore::open(dir.path()).expect("open");

        store.append(b"a").await.expect("append");
        store.append(b"b").await.expect("append");
        assert_eq!(store.len().await.expect("len"), 2);

        assert_eq!(store.pop().await.expect("pop"), Some(b"a".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), Some(b"b".to_vec()));
        assert_eq!(store.pop().await.expect("pop"), None);
    }

    #[tokio::test]
    async fn records_survive_reopening() {
        let dir = tempfile::tempdir().expect("tempdir");
        {
            let mut store = FileStore::open(dir.path()).expect("open");
            store.append(b"durable").await.expect("append");
            store.append(b"records").await.expect("append");
        }

        // A fresh store over the same directory resumes the queue in order.
        let mut reopened = FileStore::open(dir.path()).expect("reopen");
        assert_eq!(reopened.len().await.expect("len"), 2);
        assert_eq!(
            reopened.pop().await.expect("pop"),
            Some(b"durable".to_vec())
        );
        assert_eq!(
            reopened.pop().await.expect("pop"),
            Some(b"records".to_vec())
        );
    }

    #[tokio::test]
    async fn peek_reads_the_oldest_without_removing_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = FileStore::open(dir.path()).expect("open");
        store.append(b"oldest").await.expect("append");
        store.append(b"newer").await.expect("append");

        assert_eq!(store.peek().await.expect("peek"), Some(b"oldest".to_vec()));
        assert_eq!(store.len().await.expect("len"), 2);
        assert_eq!(store.pop().await.expect("pop"), Some(b"oldest".to_vec()));
    }

    #[tokio::test]
    async fn popping_removes_the_record_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store = FileStore::open(dir.path()).expect("open");
        store.append(b"once").await.expect("append");
        let _ = store.pop().await.expect("pop");

        let reopened = FileStore::open(dir.path()).expect("reopen");
        assert!(reopened.is_empty().await.expect("is_empty"));
    }
}
