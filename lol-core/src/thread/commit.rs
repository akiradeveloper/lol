use crate::{ElectionState, RaftApp, RaftCore};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

struct Thread {
    core: Arc<RaftCore>,
}
impl Thread {
    async fn run(self) {
        loop {
            while let Ok(true) = tokio::spawn({
                let core = Arc::clone(&self.core);
                async move {
                    let election_state = *core.election_state.read().await;
                    if std::matches!(election_state, ElectionState::Leader) {
                        let old_agreement = core.log.commit_index.load(Ordering::SeqCst);
                        let new_agreement = core.find_new_agreement().await.unwrap();
                        if new_agreement > old_agreement {
                            core.log
                                .advance_commit_index(new_agreement, Arc::clone(&core))
                                .await
                                .unwrap();
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            })
            .await
            {}
            // We should timeout and go to next poll because in case of one node cluster,
            // there will be no replication happen and this thread will never wake up.
            let _ = tokio::time::timeout(
                Duration::from_millis(100),
                self.core.log.replication_notify.notified(),
            )
            .await;
        }
    }
}
pub async fn run(core: Arc<RaftCore>) {
    let x = Thread { core };
    x.run().await
}
