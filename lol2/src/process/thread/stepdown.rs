use super::*;

#[derive(Clone)]
pub struct Thread {
    voter: Voter,
}

impl Thread {
    pub async fn run_once(&self) -> Result<()> {
        self.voter.try_stepdown().await?;
        Ok(())
    }

    pub fn do_loop(self) -> ThreadHandle {
        let hdl = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                self.run_once().await.ok();
            }
        })
        .abort_handle();
        ThreadHandle(hdl)
    }
}

pub fn new(voter: Voter) -> ThreadHandle {
    Thread { voter }.do_loop()
}
