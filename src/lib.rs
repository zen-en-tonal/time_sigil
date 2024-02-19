pub use errors::Error;
pub use queue::{Queue, Server};
pub use task::{Task, TaskRunner};
pub use tokio_util::sync::CancellationToken;

mod errors;
pub mod http;
mod queue;
pub mod schedule;
pub mod service;
mod task;

#[cfg(test)]
mod tests {
    use crate::{service::new, CancellationToken};
    use std::collections::VecDeque;

    #[tokio::test]
    async fn test() {
        let (runner, handler) = new(
            VecDeque::<i32>::new(),
            VecDeque::<i32>::new(),
            |x: i32| x + 1,
            4,
        );
        let cancel = CancellationToken::new();
        tokio::spawn(runner.listen(cancel.clone()));

        handler.push_task(0).await.unwrap();

        while let Ok(Some(x)) = handler.pop_result().await {
            assert_eq!(x, 1);
            cancel.cancel();
        }
    }
}
