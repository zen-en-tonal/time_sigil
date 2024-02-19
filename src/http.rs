use std::sync::Arc;

use axum::extract::Path;
use axum::routing::{delete, post};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::Job;

pub use crate::schedule::{ScheduleHandler, Uuid};
pub use crate::tasks::OneWay;
pub use axum::Router;

struct AppState<T, Q> {
    handler: OneWay<Q, T>,
    scheduler: ScheduleHandler,
}

pub fn router<T, Q>(handler: OneWay<Q, T>, scheduler: ScheduleHandler) -> Router
where
    T: Sync + Send + 'static + for<'de> Deserialize<'de> + Clone,
    Q: Sync + Send + 'static,
{
    let state = AppState { handler, scheduler };
    let app = Router::new()
        .route("/tasks/new", post(new_task))
        .route("/schedules/new", post(new_schedule))
        .route("/schedules/:id", delete(remove_schedule))
        .with_state(Arc::new(state));
    app
}

async fn new_task<T, Q>(State(h): State<Arc<AppState<T, Q>>>, Json(t): Json<T>) -> StatusCode {
    match h.handler.push(t).await {
        Ok(_) => StatusCode::ACCEPTED,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Schedule<T> {
    cron: String,
    task: T,
}

#[derive(Debug, Serialize)]
struct NewScheduleResp {
    uuid: Uuid,
}

async fn new_schedule<T, Q>(
    State(state): State<Arc<AppState<T, Q>>>,
    Json(t): Json<Schedule<T>>,
) -> (StatusCode, Json<NewScheduleResp>)
where
    T: Sync + Send + 'static + Clone,
    Q: Sync + Send + 'static,
{
    let scheduler = state.scheduler.clone();
    let job = Job::new_async(t.cron.as_str(), move |_, _| {
        let sche_h = state.handler.clone();
        let task = t.task.clone();
        Box::pin(async move {
            sche_h.push(task).await.unwrap();
        })
    })
    .unwrap();
    let uuid = scheduler.add(job).await.unwrap();
    return (StatusCode::CREATED, Json(NewScheduleResp { uuid }));
}

async fn remove_schedule<T, Q>(
    State(state): State<Arc<AppState<T, Q>>>,
    Path(id): Path<Uuid>,
) -> StatusCode
where
    T: Sync + Send + 'static,
    Q: Sync + Send + 'static,
{
    let scheduler = state.scheduler.clone();
    scheduler.remove(id).await.unwrap();
    return StatusCode::OK;
}
