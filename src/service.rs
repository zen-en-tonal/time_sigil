use tokio_util::sync::CancellationToken;

use crate::{queue, Error, Queue, Server, Task, TaskRunner};

pub fn new<T, Q, A, B, F>(
    task_q: A,
    res_q: B,
    task: F,
    num_task: usize,
) -> (Runner<A, T, B, Q, F>, Handler<T, Q>)
where
    A: Queue<T>,
    B: Queue<Q>,
    F: Task<T, Q> + Clone,
{
    let (t_s, t_h) = task_q.into_server(10);
    let (r_s, r_h) = res_q.into_server(10);
    let t = task.into_runner(t_h.clone(), r_h.clone());
    let mut tasks = vec![];
    for _ in 0..num_task {
        tasks.push(t.clone());
    }
    let runner = Runner {
        task_q: t_s,
        res_q: r_s,
        tasks,
    };
    let handler = Handler {
        task_h: t_h.clone(),
        res_h: r_h.clone(),
    };
    (runner, handler)
}

pub struct Runner<Q, A, T, B, F> {
    task_q: Server<Q, A>,
    res_q: Server<T, B>,
    tasks: Vec<TaskRunner<A, B, F>>,
}

impl<Q, A, T, B, F> Runner<Q, A, T, B, F>
where
    T: Queue<B> + Sync + Send + 'static,
    Q: Queue<A> + Sync + Send + 'static,
    F: Task<A, B> + Clone + Sync + Send + 'static,
    A: Sync + Send + 'static,
    B: Sync + Send + 'static,
{
    pub async fn listen(self, cancel: CancellationToken) -> Result<(), Error> {
        tokio::spawn(self.listening(cancel));
        Ok(())
    }

    async fn listening(self, cancel: CancellationToken) -> Result<(), Error> {
        let mut set = tokio::task::JoinSet::new();
        set.spawn(self.task_q.listen(cancel.clone()));
        set.spawn(self.res_q.listen(cancel.clone()));
        for t in self.tasks {
            set.spawn(t.listen(cancel.clone()));
        }

        tokio::select! {
            _ = cancel.cancelled() => Ok(())
        }
    }
}

#[derive(Debug)]
pub struct Handler<T, Q> {
    task_h: queue::Handler<T>,
    res_h: queue::Handler<Q>,
}

impl<T, Q> Handler<T, Q> {
    pub async fn push_task(&self, task: T) -> Result<(), Error> {
        self.task_h.enque(task).await
    }

    pub async fn pop_result(&self) -> Result<Option<Q>, Error> {
        self.res_h.deque().await
    }
}

impl<T, Q> Clone for Handler<T, Q> {
    fn clone(&self) -> Self {
        Self {
            task_h: self.task_h.clone(),
            res_h: self.res_h.clone(),
        }
    }
}
