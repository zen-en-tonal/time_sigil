use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_cron_scheduler::JobScheduler;
pub use tokio_cron_scheduler::{Job, JobSchedulerError};

pub struct Schedule {
    inner: JobScheduler,
    rx: Receiver<Job>,
}

impl Schedule {
    pub async fn new() -> (Self, Sender<Job>) {
        let sched = JobScheduler::new().await.unwrap();
        let (tx, rx) = mpsc::channel(10);
        (
            Self {
                inner: sched.into(),
                rx,
            },
            tx,
        )
    }

    pub async fn start(self) -> Result<(), JobSchedulerError> {
        self.inner.start().await?;
        tokio::spawn(self.listen_add_sched());
        Ok(())
    }

    async fn listen_add_sched(mut self) {
        loop {
            while let Some(job) = self.rx.recv().await {
                self.inner.add(job);
            }
        }
    }
}
