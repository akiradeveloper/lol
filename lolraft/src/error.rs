use super::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("log state is broken")]
    BadLogState,
    #[error("replication stream is broken")]
    BadReplicationStream,
    #[error("snapshot chunk is broken. error={0}")]
    BadSnapshotChunk(#[from] tonic::Status),
    #[error("entry not found at index {0}")]
    EntryNotFound(u64),
    #[error("leader is unknown")]
    LeaderUnknown,
    #[error("peer (node_id={0}) not found")]
    PeerNotFound(NodeId),
    #[error("process not found (shard_id={0})")]
    ProcessNotFound(ShardId),
}
