use super::{Ballot, Entry};
use crate::{Clock, Id, Index};
use rocksdb::{ColumnFamilyDescriptor, IteratorMode, Options, DB};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const CF_ENTRIES: &str = "entries";
const CF_TAGS: &str = "tags";
const CF_CTRL: &str = "ctrl";
const BALLOT: &str = "ballot";
const CMP: &str = "index_asc";

#[derive(serde::Serialize, serde::Deserialize)]
struct EntryB {
    prev_clock: (u64, u64),
    this_clock: (u64, u64),
    command: bytes::Bytes,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct BallotB {
    term: u64,
    voted_for: Option<Id>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct SnapshotIndexB(u64);

impl From<Vec<u8>> for Entry {
    fn from(x: Vec<u8>) -> Self {
        let x: EntryB = bincode::deserialize(&x).unwrap();
        Entry {
            prev_clock: Clock {
                term: x.prev_clock.0,
                index: x.prev_clock.1,
            },
            this_clock: Clock {
                term: x.this_clock.0,
                index: x.this_clock.1,
            },
            command: x.command.into(),
        }
    }
}
impl Into<Vec<u8>> for Entry {
    fn into(self) -> Vec<u8> {
        let x = EntryB {
            prev_clock: (self.prev_clock.term, self.prev_clock.index),
            this_clock: (self.this_clock.term, self.this_clock.index),
            command: self.command,
        };
        bincode::serialize(&x).unwrap()
    }
}

impl From<Vec<u8>> for Ballot {
    fn from(x: Vec<u8>) -> Self {
        let x: BallotB = bincode::deserialize(&x).unwrap();
        Ballot {
            cur_term: x.term,
            voted_for: x.voted_for,
        }
    }
}
impl Into<Vec<u8>> for Ballot {
    fn into(self) -> Vec<u8> {
        let x = BallotB {
            term: self.cur_term,
            voted_for: self.voted_for,
        };
        bincode::serialize(&x).unwrap()
    }
}

impl From<Vec<u8>> for SnapshotIndexB {
    fn from(x: Vec<u8>) -> Self {
        bincode::deserialize(&x).unwrap()
    }
}
impl Into<Vec<u8>> for SnapshotIndexB {
    fn into(self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct IndexKey(u64);
fn encode_index(i: Index) -> Vec<u8> {
    bincode::serialize(&IndexKey(i)).unwrap()
}
fn decode_index(s: &[u8]) -> Index {
    let x: IndexKey = bincode::deserialize(s).unwrap();
    x.0
}
fn comparator_fn(x: &[u8], y: &[u8]) -> Ordering {
    let x: Index = decode_index(x);
    let y: Index = decode_index(y);
    x.cmp(&y)
}

pub struct StorageBuilder {
    path: PathBuf,
}
impl StorageBuilder {
    pub fn new(path: &Path) -> Self {
        StorageBuilder {
            path: path.to_owned(),
        }
    }
    pub fn destory(&self) {
        let opts = Options::default();
        DB::destroy(&opts, &self.path).unwrap();
    }
    pub fn create(&self) {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        let mut opts = Options::default();
        opts.set_comparator(CMP, comparator_fn);
        let cf_descs = vec![
            ColumnFamilyDescriptor::new(CF_ENTRIES, opts),
            ColumnFamilyDescriptor::new(CF_TAGS, Options::default()),
            ColumnFamilyDescriptor::new(CF_CTRL, Options::default()),
        ];
        let db = DB::open_cf_descriptors(&db_opts, &self.path, cf_descs).unwrap();

        // let mut opts = Options::default();
        // opts.set_comparator("by_index_key", comparator_fn);
        // db.create_cf(CF_ENTRIES, &opts).unwrap();
        // db.create_cf(CF_CTRL, &Options::default()).unwrap();

        let initial_ballot = Ballot {
            cur_term: 0,
            voted_for: None,
        };
        let cf = db.cf_handle(CF_CTRL).unwrap();
        let b: Vec<u8> = initial_ballot.into();
        db.put_cf(&cf, BALLOT, b).unwrap();
    }
    fn open_db(&self) -> DB {
        let db_opts = Options::default();
        let mut opts = Options::default();
        opts.set_comparator(CMP, comparator_fn);
        let cf_descs = vec![
            ColumnFamilyDescriptor::new(CF_ENTRIES, opts),
            ColumnFamilyDescriptor::new(CF_TAGS, Options::default()),
            ColumnFamilyDescriptor::new(CF_CTRL, Options::default()),
        ];
        DB::open_cf_descriptors(&db_opts, &self.path, cf_descs).unwrap()
    }
    pub fn open(&self) -> Storage {
        let db = self.open_db();
        Storage::new(db)
    }
}

pub struct Storage {
    db: DB,
}
impl Storage {
    fn new(db: DB) -> Self {
        Self { db }
    }
}
use anyhow::Result;
#[async_trait::async_trait]
impl super::RaftStorage for Storage {
    async fn list_tags(&self) -> Result<BTreeSet<Index>> {
        let cf = self.db.cf_handle(CF_TAGS).unwrap();
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);
        let mut r = BTreeSet::new();
        for (k, _) in iter {
            r.insert(decode_index(&k));
        }
        Ok(r)
    }
    async fn delete_tag(&self, i: Index) -> Result<()> {
        let cf = self.db.cf_handle(CF_TAGS).unwrap();
        self.db.delete_cf(&cf, encode_index(i))?;
        Ok(())
    }
    async fn put_tag(&self, i: Index, x: crate::SnapshotTag) -> Result<()> {
        let cf = self.db.cf_handle(CF_TAGS).unwrap();
        self.db.put_cf(&cf, encode_index(i), x)?;
        Ok(())
    }
    async fn get_tag(&self, i: Index) -> Result<Option<crate::SnapshotTag>> {
        let cf = self.db.cf_handle(CF_TAGS).unwrap();
        let b: Option<Vec<u8>> = self.db.get_cf(&cf, encode_index(i))?;
        Ok(b.map(|x| x.into()))
    }
    async fn delete_before(&self, r: Index) -> Result<()> {
        let cf = self.db.cf_handle(CF_ENTRIES).unwrap();
        self.db
            .delete_range_cf(cf, encode_index(0), encode_index(r))?;
        Ok(())
    }
    async fn get_last_index(&self) -> Result<Index> {
        let cf = self.db.cf_handle(CF_ENTRIES).unwrap();
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek_to_last();
        // The iterator is empty
        if !iter.valid() {
            return Ok(0);
        }
        let key = iter.key().unwrap();
        let v = decode_index(key);
        Ok(v)
    }
    async fn insert_entry(&self, i: Index, e: Entry) -> Result<()> {
        let cf = self.db.cf_handle(CF_ENTRIES).unwrap();
        let b: Vec<u8> = e.into();
        self.db.put_cf(&cf, encode_index(i), b)?;
        Ok(())
    }
    async fn get_entry(&self, i: Index) -> Result<Option<Entry>> {
        let cf = self.db.cf_handle(CF_ENTRIES).unwrap();
        let b: Option<Vec<u8>> = self.db.get_cf(&cf, encode_index(i))?;
        Ok(b.map(|x| x.into()))
    }
    async fn save_ballot(&self, v: Ballot) -> Result<()> {
        let cf = self.db.cf_handle(CF_CTRL).unwrap();
        let b: Vec<u8> = v.into();
        self.db.put_cf(&cf, BALLOT, b)?;
        Ok(())
    }
    async fn load_ballot(&self) -> Result<Ballot> {
        let cf = self.db.cf_handle(CF_CTRL).unwrap();
        let b = self.db.get_cf(&cf, BALLOT)?.unwrap();
        let v = b.into();
        Ok(v)
    }
}

#[tokio::test]
async fn test_rocksdb_storage() -> Result<()> {
    let _ = std::fs::create_dir("/tmp/lol");
    let path = Path::new("/tmp/lol/disk1.db");
    let builder = StorageBuilder::new(&path);
    builder.destory();
    builder.create();
    let s = builder.open();

    super::test_storage(s).await?;

    builder.destory();
    Ok(())
}

#[tokio::test]
async fn test_rocksdb_persistency() -> Result<()> {
    use super::RaftStorage;
    use crate::Command;
    use std::collections::HashSet;

    let _ = std::fs::create_dir("/tmp/lol");
    let path = Path::new("/tmp/lol/disk2.db");
    let builder = StorageBuilder::new(&path);
    builder.destory();
    builder.create();
    let s = builder.open();

    let e = Entry {
        prev_clock: Clock { term: 0, index: 0 },
        this_clock: Clock { term: 0, index: 0 },
        command: Command::serialize(&Command::Noop),
    };
    let sn = Entry {
        prev_clock: Clock { term: 0, index: 0 },
        this_clock: Clock { term: 0, index: 0 },
        command: Command::serialize(&Command::Snapshot {
            membership: HashSet::new(),
        }),
    };
    let tag: crate::SnapshotTag = vec![].into();

    s.put_tag(1, tag.clone()).await?;
    s.insert_entry(1, sn.clone()).await?;
    s.insert_entry(2, e.clone()).await?;
    s.insert_entry(3, e.clone()).await?;
    s.insert_entry(4, e.clone()).await?;
    s.put_tag(3, tag.clone()).await?;
    s.insert_entry(3, sn.clone()).await?;

    drop(s);

    let s = builder.open();
    assert_eq!(
        s.load_ballot().await?,
        Ballot {
            cur_term: 0,
            voted_for: None
        }
    );
    assert!(s.get_tag(1).await?.is_some());
    assert!(s.get_tag(2).await?.is_none());
    assert!(s.get_tag(3).await?.is_some());
    assert_eq!(super::find_last_snapshot_index(&s).await?, Some(3));
    assert_eq!(s.get_last_index().await?, 4);

    drop(s);

    builder.destory();
    Ok(())
}
