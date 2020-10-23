use super::notification;
use crate::{RaftApp, RaftCore};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

struct Thread<A: RaftApp> {
    core: Arc<RaftCore<A>>,
    subscriber: notification::Subscriber,
}
impl<A: RaftApp> Thread<A> {
    async fn run(self) {
        loop {
            while let Ok(true) = tokio::spawn({
                let core = Arc::clone(&self.core);
                async move {
                    let need_work = core.log.last_applied.load(Ordering::SeqCst)
                        < core.log.commit_index.load(Ordering::SeqCst);
                    if need_work {
                        core.log.advance_last_applied(Arc::clone(&core)).await.unwrap();
                    }
                    need_work
                }
            })
            .await
            {}
            let _ = tokio::time::timeout(Duration::from_millis(100), self.subscriber.wait()).await;
        }
    }
}
pub async fn run<A: RaftApp>(core: Arc<RaftCore<A>>) {
    let subscriber = core.log.commit_notification.lock().await.subscribe();
    let x = Thread { core, subscriber };
    x.run().await
}
