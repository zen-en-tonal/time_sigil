use tokio::sync::{
    mpsc::{self},
    oneshot,
};
use tokio_util::sync::CancellationToken;

pub struct Server<Q, T> {
    queue: Q,
    rx: mpsc::Receiver<Command<T>>,
    shutdown: CancellationToken,
}

pub trait Queue<T> {
    fn enque(&mut self, item: T);
    fn deque(&mut self) -> Option<T>;
}

impl<Q, T> Server<Q, T> {
    pub fn new(
        queue: Q,
        channels: usize,
        shutdown: CancellationToken,
    ) -> (Server<Q, T>, Handler<T>) {
        let (tx, rx) = mpsc::channel(channels);
        let handler = Handler { tx };
        let server = Server {
            queue,
            rx,
            shutdown,
        };
        (server, handler)
    }
}

impl<Q, T> Server<Q, T>
where
    Q: Queue<T>,
{
    pub async fn listen(self) {
        let token = self.shutdown.clone();
        tokio::select! {
            _ = self.listening() => {},
            _ = token.cancelled() => {}
        }
    }

    async fn listening(mut self) {
        loop {
            if let Some(recv) = self.rx.recv().await {
                match recv {
                    Command::Enque(item) => {
                        self.queue.enque(item);
                    }
                    Command::Deque(tx) => {
                        let item = self.queue.deque();
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

#[derive(Debug)]
pub struct Handler<T> {
    tx: mpsc::Sender<Command<T>>,
}

impl<T> Clone for Handler<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<T> Handler<T> {
    pub async fn enque(&self, item: T) -> Result<(), String> {
        self.tx.send(Command::Enque(item)).await.expect("");
        Ok(())
    }

    pub async fn deque(&self) -> Result<Option<T>, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(Command::Deque(tx)).await.expect("");
        Ok(rx.await.expect(""))
    }
}
