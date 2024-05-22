use super::*;

use spin::Mutex;
use std::collections::HashMap;

pub struct HeartbeatBuffer {
    buf: HashMap<LaneId, request::Heartbeat>,
}
impl HeartbeatBuffer {
    pub fn new() -> Self {
        Self {
            buf: HashMap::new(),
        }
    }

    pub fn push(&mut self, lane_id: LaneId, req: request::Heartbeat) {
        self.buf.insert(lane_id, req);
    }

    fn drain(&mut self) -> HashMap<LaneId, request::Heartbeat> {
        self.buf.drain().collect()
    }
}

pub async fn run(
    buf: Arc<Mutex<HeartbeatBuffer>>,
    mut cli: raft::RaftClient,
    self_node_id: NodeId,
) {
    loop {
        tokio::time::sleep(Duration::from_millis(300)).await;

        let states = {
            let mut buf = buf.lock();
            let heartbeats = buf.drain();

            let mut out = HashMap::new();
            for (lane_id, heartbeat) in heartbeats {
                let state = raft::LeaderCommitState {
                    leader_term: heartbeat.leader_term,
                    leader_commit_index: heartbeat.leader_commit_index,
                };
                out.insert(lane_id, state);
            }
            out
        };

        let req = raft::Heartbeat {
            leader_id: self_node_id.to_string(),
            leader_commit_states: states,
        };
        cli.send_heartbeat(req).await.ok();
    }
}
