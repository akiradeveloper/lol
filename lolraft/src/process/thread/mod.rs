use super::*;

use tokio::task::AbortHandle;

pub mod advance_commit;
pub mod advance_kern;
pub mod advance_snapshot;
pub mod advance_user;
pub mod election;
pub mod heartbeat;
pub mod log_compaction;
pub mod query_execution;
pub mod replication;
pub mod snapshot_deleter;
pub mod stepdown;

/// Wrapper around a `AbortHandle` that aborts it is dropped.
pub struct ThreadHandle(AbortHandle);
impl Drop for ThreadHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct EventProducer<T> {
    inner: Arc<Notify>,
    phantom: PhantomData<T>,
}
impl<T> EventProducer<T> {
    pub fn push_event(&self, _: T) {
        self.inner.notify_one();
    }
}

#[derive(Clone)]
pub struct EventConsumer<T> {
    inner: Arc<Notify>,
    phantom: PhantomData<T>,
}
impl<T> EventConsumer<T> {
    /// Return if events are produced or timeout.
    pub async fn consume_events(&self, timeout: Duration) {
        tokio::time::timeout(timeout, self.inner.notified())
            .await
            .ok();
    }
}

pub fn notify<T>() -> (EventProducer<T>, EventConsumer<T>) {
    let inner = Arc::new(Notify::new());
    (
        EventProducer {
            inner: inner.clone(),
            phantom: PhantomData,
        },
        EventConsumer {
            inner,
            phantom: PhantomData,
        },
    )
}

#[derive(Clone)]
pub struct QueueEvent;

#[derive(Clone)]
pub struct ReplicationEvent;

#[derive(Clone)]
pub struct CommitEvent;

#[derive(Clone)]
pub struct KernEvent;
