mod fn_task;
mod runner;

pub use fn_task::fn_task;
pub use runner::{IntoTaskRunner, TaskRunner};

pub trait Task<T, Q> {
    fn run(&self, req: T) -> Q;
}
