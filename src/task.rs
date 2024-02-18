use tokio_util::sync::CancellationToken;

use crate::queue::Handler;

pub trait Task<T, Q> {
    fn run(&self, req: T) -> Q;
}

impl<F: Fn(T) -> Q, T, Q> Task<T, Q> for F {
    fn run(&self, req: T) -> Q {
        (self)(req)
    }
}

pub async fn task<T, Q>(
    t: impl Task<T, Q>,
    request_queue: Handler<T>,
    result_queue: Handler<Q>,
    cancel: CancellationToken,
) {
    tokio::select! {
        _ = work(t, request_queue, result_queue) => {},
        _ = cancel.cancelled() => {}
    }
}

async fn work<T, Q>(t: impl Task<T, Q>, request_queue: Handler<T>, result_queue: Handler<Q>) {
    loop {
        if let Ok(Some(req)) = request_queue.deque().await {
            let res = t.run(req);
            let _ = result_queue.enque(res).await;
        }
    }
}
