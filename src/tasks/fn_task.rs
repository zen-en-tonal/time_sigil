use crate::Task;

#[derive(Debug, Clone)]
pub struct FnTask<T> {
    inner: T,
}

pub fn fn_task<F, T, Q>(f: F) -> FnTask<F>
where
    F: Fn(T) -> Q + Clone,
{
    FnTask { inner: f }
}

impl<F: Fn(T) -> Q + Clone, T, Q> Task<T, Q> for FnTask<F> {
    fn run(&self, req: T) -> Q {
        (self.inner)(req)
    }
}
