use crate::{errors::Error, Task};
use tokio_util::sync::CancellationToken;

use super::OneWay;

pub trait IntoTaskRunner<T, Q>: Sized {
    fn into_task_runner(self, propagate: OneWay<T, Q>) -> TaskRunner<T, Q, Self> {
        TaskRunner {
            task: self,
            propagate,
        }
    }
}

impl<F, T, Q> IntoTaskRunner<T, Q> for F
where
    F: Task<T, Q> + Sized,
{
    fn into_task_runner(self, propagate: OneWay<T, Q>) -> TaskRunner<T, Q, Self> {
        TaskRunner {
            task: self,
            propagate,
        }
    }
}

#[derive(Debug)]
pub struct TaskRunner<T, Q, F> {
    task: F,
    propagate: OneWay<T, Q>,
}

impl<T, Q, F: Clone> Clone for TaskRunner<T, Q, F> {
    fn clone(&self) -> Self {
        Self {
            task: self.task.clone(),
            propagate: self.propagate.clone(),
        }
    }
}

impl<T, Q, F: Task<T, Q>> TaskRunner<T, Q, F>
where
    T: Sync + Send + 'static,
    Q: Sync + Send + 'static,
    F: Sync + Send + 'static + Clone,
{
    pub async fn listen(self, num_workers: usize, cancel: CancellationToken) -> Result<(), Error> {
        let mut set = tokio::task::JoinSet::new();
        for _ in 0..num_workers {
            set.spawn(self.clone().work());
        }
        tokio::select! {
            _ = cancel.cancelled() => {
                set.abort_all();
                Ok(())
            }
        }
    }

    async fn work(self) -> Result<(), Error> {
        loop {
            if let Ok(Some(req)) = self.propagate.pull().await {
                let res = self.task.run(req);
                match self.propagate.push(res).await {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
