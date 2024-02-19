use time_sigil::{
    schedule::{Job, Schedule},
    *,
};

#[tokio::main]
async fn main() {
    let (runner, handler) = service::new_inmemory(fn_task(|t| t));

    let token = CancellationToken::new();

    let (sched, sched_handler) = Schedule::new().await;

    runner.listen(1, token.clone()).await.unwrap();
    sched.start().await.unwrap();

    let h = handler.clone();
    let id = sched_handler
        .add(
            Job::new_async("1/10 * * * * *", move |uuid, _| {
                let handler = h.clone();
                Box::pin(async move {
                    handler
                        .push(Task {
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
    println!("job {} was added.", id);

    loop {
        if let Ok(Some(x)) = handler.pull().await {
            println!("{:?}", x);
            break;
        }
    }
    token.cancel();
}

#[derive(Debug)]
#[allow(dead_code)]
struct Task {
    uuid: String,
    msg: String,
}
