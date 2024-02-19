use tokio_util::sync::CancellationToken;

use crate::{errors::Error, queue::Handler};

pub trait Task<T, Q>: Sized {
    fn run(&self, req: T) -> Q;

    fn into_runner(self, req_q: Handler<T>, res_q: Handler<Q>) -> TaskRunner<T, Q, Self> {
        TaskRunner {
            task: self,
            req_q,
            res_q,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnTask<T> {
    inner: T,
}

pub fn fn_task<F, T, Q>(f: F) -> FnTask<F>
where
    F: Fn(T) -> Q + Clone,
{
    FnTask { inner: f }
}

impl<F: Fn(T) -> Q + Clone, T, Q> Task<T, Q> for FnTask<F> {
    fn run(&self, req: T) -> Q {
        (self.inner)(req)
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
