use std::collections::VecDeque;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Default)]
struct Queue<T> {
    inner: VecDeque<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Self {
            inner: VecDeque::new(),
        }
    }
}

pub struct Server<T> {
    queue: Queue<T>,
    rx: mpsc::Receiver<Command<T>>,
}

impl<T> Server<T> {
    pub fn new(channels: usize) -> (Server<T>, Handler<T>) {
        let queue = Queue::<T>::new();
        let (tx, rx) = mpsc::channel(channels);
        let handler = Handler { tx };
        let server = Server { queue, rx };
        (server, handler)
    }

    pub async fn run(mut self) {
        loop {
            if let Some(recv) = self.rx.recv().await {
                match recv {
                    Command::Enque(item) => {
                        self.queue.inner.push_back(item);
                    }
                    Command::Deque(tx) => {
                        let item = self.queue.inner.pop_front();
                        let _ = tx.send(item);
                    }
                }
            }
        }
    }
}

enum Command<T> {
    Enque(T),
    Deque(oneshot::Sender<Option<T>>),
}

#[derive(Debug, Clone)]
pub struct Handler<T> {
    tx: mpsc::Sender<Command<T>>,
}

impl<T> Handler<T> {
    pub async fn enque(&self, item: T) {
        self.tx.send(Command::Enque(item)).await.expect("msg");
    }

    pub async fn deque(&self) -> Option<T> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Command::Deque(tx))
            .await
            .expect("falid to send deque command");
        rx.await.expect("msg")
    }
}
