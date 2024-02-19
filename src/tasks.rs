mod fn_task;
mod runner;

pub use fn_task::fn_task;
pub use runner::{IntoTaskRunner, TaskRunner};

use crate::{queue::Handler, Error};

pub trait Task<T, Q> {
    fn run(&self, req: T) -> Q;
}

#[derive(Debug)]
pub struct OneWay<TPull, TPush> {
    pull: Handler<TPull>,
    push: Handler<TPush>,
}

impl<TPull, TPush> OneWay<TPull, TPush> {
    pub fn new(pull: Handler<TPull>, push: Handler<TPush>) -> Self {
        Self { pull, push }
    }

    pub async fn pull(&self) -> Result<Option<TPull>, Error> {
        self.pull.deque().await
    }

    pub async fn push(&self, res: TPush) -> Result<(), Error> {
        self.push.enque(res).await
    }
}

impl<T, Q> Clone for OneWay<T, Q> {
    fn clone(&self) -> Self {
        Self {
            pull: self.pull.clone(),
            push: self.push.clone(),
        }
    }
}
