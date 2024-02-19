use crate::{errors::Error, queue::Handler, Task};
use tokio_util::sync::CancellationToken;

pub trait IntoTaskRunner<T, Q>: Sized {
    fn into_task_runner(
        self,
        req_queue: Handler<T>,
        res_queue: Handler<Q>,
    ) -> TaskRunner<T, Q, Self> {
        TaskRunner {
            task: self,
            req_q: req_queue,
            res_q: res_queue,
        }
    }
}

impl<F, T, Q> IntoTaskRunner<T, Q> for F
where
    F: Task<T, Q> + Sized,
{
    fn into_task_runner(
        self,
        req_queue: Handler<T>,
        res_queue: Handler<Q>,
    ) -> TaskRunner<T, Q, Self> {
        TaskRunner {
            task: self,
            req_q: req_queue,
            res_q: res_queue,
        }
    }
}

#[derive(Debug)]
pub struct TaskRunner<T, Q, F> {
    task: F,
    req_q: Handler<T>,
    res_q: Handler<Q>,
}

impl<T, Q, F: Clone> Clone for TaskRunner<T, Q, F> {
    fn clone(&self) -> Self {
        Self {
            task: self.task.clone(),
            req_q: self.req_q.clone(),
            res_q: self.res_q.clone(),
        }
    }
}

impl<T, Q, F: Task<T, Q>> TaskRunner<T, Q, F> {
    pub async fn listen(self, cancel: CancellationToken) -> Result<(), Error> {
        let req_q = self.req_q.clone();
        let res_q = self.res_q.clone();
        self.listening(req_q, res_q, cancel).await
    }

    async fn listening(
        self,
        request_queue: Handler<T>,
        result_queue: Handler<Q>,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        tokio::select! {
            res = self.work(request_queue, result_queue) => match res {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            },
            _ = cancel.cancelled() => Ok(())
        }
    }

    async fn work(self, request_queue: Handler<T>, result_queue: Handler<Q>) -> Result<(), Error> {
        loop {
            if let Ok(Some(req)) = request_queue.deque().await {
                let res = self.task.run(req);
                match result_queue.enque(res).await {
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
