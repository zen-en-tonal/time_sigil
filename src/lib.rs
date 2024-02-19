pub use errors::Error;
pub use queue::{Queue, Server};
pub use tasks::{fn_task, IntoTaskRunner, Task, TaskRunner};
pub use tokio_util::sync::CancellationToken;

mod errors;
pub mod http;
mod queue;
pub mod schedule;
pub mod service;
mod tasks;

#[cfg(test)]
mod tests {
    use crate::{fn_task, service::new_inmemory, CancellationToken};

    #[tokio::test]
    async fn test() {
        let (runner, handler) = new_inmemory(fn_task(|x: i32| x + 1));
        let cancel = CancellationToken::new();
        tokio::spawn(runner.listen(4, cancel.clone()));

        handler.push(0).await.unwrap();

        while let Ok(Some(x)) = handler.pull().await {
            assert_eq!(x, 1);
            cancel.cancel();
        }
    }
}
