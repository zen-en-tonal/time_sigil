mod queue;
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::post, Router};
use queue::Handler;
use reqwest::header::CONTENT_TYPE;

struct AppState {
    task_handler: Handler<String>,
}

#[tokio::main]
async fn main() {
    let (task_queue, task_handler) = queue::Server::<String>::new(4);

    tokio::spawn(task_queue.run());
    tokio::spawn(task(task_handler.clone()));

    let state = Arc::new(AppState { task_handler });

    let app = Router::new()
        .route("/task", post(queue_task))
        .with_state(state);

    let listner = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listner, app).await.unwrap();
}

async fn task(handler: Handler<String>) {
    let client = reqwest::Client::new();
    loop {
        if let Some(req) = handler.deque().await {
            let req = client
                .post("http://yagura:8000")
                .body(req)
                .header(CONTENT_TYPE, "application/json")
                .build()
                .expect("");
            let resp = client.execute(req).await.expect("");
            println!("{}", resp.text().await.expect("msg"))
        }
    }
}

async fn queue_task(State(s): State<Arc<AppState>>, b: String) -> StatusCode {
    let _ = s.task_handler.enque(b).await;
    StatusCode::ACCEPTED
}
