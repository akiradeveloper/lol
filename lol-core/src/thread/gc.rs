use crate::{RaftApp, RaftCore};
use std::sync::Arc;
use std::time::Duration;

struct Thread<A: RaftApp> {
    core: Arc<RaftCore<A>>,
}
impl<A: RaftApp> Thread<A> {
    async fn run(self) {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let core = Arc::clone(&self.core);
            let f = async move {
                core.log.run_gc(Arc::clone(&core)).await.unwrap();
            };
            let _ = tokio::spawn(f).await;
        }
    }
}
pub async fn run<A: RaftApp>(core: Arc<RaftCore<A>>) {
    let x = Thread { core };
    x.run().await
}
