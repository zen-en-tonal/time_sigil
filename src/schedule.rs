use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};
use tokio_cron_scheduler::JobScheduler;
pub use tokio_cron_scheduler::{Job, JobSchedulerError};
pub use uuid::Uuid;

use crate::Error;

enum ScheduleCommand {
    New(Job, oneshot::Sender<Uuid>),
    Remove(Uuid),
}

pub struct Schedule {
    inner: JobScheduler,
    rx: Receiver<ScheduleCommand>,
}

#[derive(Debug)]
pub struct ScheduleHandler {
    tx: Sender<ScheduleCommand>,
}

impl Clone for ScheduleHandler {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl ScheduleHandler {
    pub async fn add(&self, job: Job) -> Result<Uuid, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(ScheduleCommand::New(job, tx))
            .await
            .map_err(|_| Error::ChannelClosed)?;
        rx.await.map_err(|_| Error::ChannelClosed)
    }

    pub async fn remove(&self, id: Uuid) -> Result<(), Error> {
        self.tx
            .send(ScheduleCommand::Remove(id))
            .await
            .map_err(|_| Error::ChannelClosed)
    }
}

impl Schedule {
    pub async fn new() -> (Self, ScheduleHandler) {
        let sched = JobScheduler::new().await.unwrap();
        let (tx, rx) = mpsc::channel(10);
        (
            Self {
                inner: sched.into(),
                rx,
            },
            ScheduleHandler { tx },
        )
    }

    pub async fn start(self) -> Result<(), JobSchedulerError> {
        self.inner.start().await?;
        tokio::spawn(self.listen_add_sched());
        Ok(())
    }

    async fn listen_add_sched(mut self) {
        loop {
            while let Some(c) = self.rx.recv().await {
                match c {
                    ScheduleCommand::New(job, tx) => {
                        let uuid = self.inner.add(job).await.unwrap();
                        tx.send(uuid).unwrap();
                    }
                    ScheduleCommand::Remove(id) => {
                        self.inner.remove(&id).await.unwrap();
                    }
                }
            }
        }
    }
}
