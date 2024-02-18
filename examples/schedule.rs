use std::collections::VecDeque;
use tokio_cron_scheduler::{Job, JobScheduler};
use tr::*;

#[tokio::main]
async fn main() {
    let (runner, handler) = service::new(VecDeque::<Task>::new(), VecDeque::<Task>::new(), task, 1);

    let token = CancellationToken::new();

    let sched = JobScheduler::new().await.unwrap();
    let sched_handler = handler.clone();
    sched
        .add(
            Job::new_async("1/10 * * * * *", move |uuid, _| {
                let sche_h = sched_handler.clone();
                Box::pin(async move {
                    sche_h
                        .push_task(Task {
                            uuid: uuid.to_string(),
                            msg: "hello".to_string(),
                        })
                        .await
                        .unwrap();
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    runner.listen(token).await.unwrap();
    sched.start().await.unwrap();

    loop {
        while let Ok(Some(x)) = handler.pop_result().await {
            println!("{:?}", x)
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Task {
    uuid: String,
    msg: String,
}

fn task(t: Task) -> Task {
    t
}
