use std::collections::VecDeque;

use tokio_util::sync::CancellationToken;

use crate::{tasks::OneWay, Error, IntoTaskRunner, Queue, Server, Task, TaskRunner};

pub fn new_inmemory<T, Q, F>(task: F) -> (Runner<VecDeque<T>, T, VecDeque<Q>, Q, F>, OneWay<Q, T>)
where
    F: IntoTaskRunner<T, Q> + Clone,
{
    new(VecDeque::new(), VecDeque::new(), task)
}

pub fn new<T, Q, A, B, F>(task_q: A, res_q: B, task: F) -> (Runner<A, T, B, Q, F>, OneWay<Q, T>)
where
    A: Queue<T>,
    B: Queue<Q>,
    F: IntoTaskRunner<T, Q> + Clone,
{
    let (t_s, t_h) = task_q.into_server(10);
    let (r_s, r_h) = res_q.into_server(10);
    let task_oneway = OneWay::new(t_h.clone(), r_h.clone());
    let task = task.into_task_runner(task_oneway);
    let runner = Runner {
        task_q: t_s,
        res_q: r_s,
        task,
    };
    (runner, OneWay::new(r_h.clone(), t_h.clone()))
}

pub struct Runner<Q, A, T, B, F> {
    task_q: Server<Q, A>,
    res_q: Server<T, B>,
    task: TaskRunner<A, B, F>,
}

impl<Q, A, T, B, F> Runner<Q, A, T, B, F>
where
    T: Queue<B> + Sync + Send + 'static,
    Q: Queue<A> + Sync + Send + 'static,
    F: Task<A, B> + Clone + Sync + Send + 'static,
    A: Sync + Send + 'static,
    B: Sync + Send + 'static,
{
    pub async fn listen(self, num_task: usize, cancel: CancellationToken) -> Result<(), Error> {
        tokio::spawn(self.listening(num_task, cancel));
        Ok(())
    }

    async fn listening(self, num_task: usize, cancel: CancellationToken) -> Result<(), Error> {
        let mut set = tokio::task::JoinSet::new();
        set.spawn(self.task_q.listen(cancel.clone()));
        set.spawn(self.res_q.listen(cancel.clone()));
        set.spawn(self.task.listen(num_task, cancel.clone()));

        tokio::select! {
            _ = cancel.cancelled() => Ok(())
        }
    }
}
