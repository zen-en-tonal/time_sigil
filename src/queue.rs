use std::collections::VecDeque;

use tokio::sync::{
    mpsc::{self},
    oneshot,
};
use tokio_util::sync::CancellationToken;

use crate::errors::Error;

pub trait Queue<T>: Sized {
    fn enque(&mut self, item: T);
    fn deque(&mut self) -> Option<T>;
    fn into_server(self, channels: usize) -> (Server<Self, T>, Handler<T>) {
        Server::new(self, channels)
    }
}

impl<T> Queue<T> for VecDeque<T> {
    fn enque(&mut self, item: T) {
        self.push_back(item)
    }

    fn deque(&mut self) -> Option<T> {
        self.pop_front()
    }
}

#[derive(Debug)]
pub struct Server<Q, T> {
    queue: Q,
    rx: mpsc::Receiver<Command<T>>,
}

impl<Q, T> Server<Q, T> {
    fn new(queue: Q, channels: usize) -> (Server<Q, T>, Handler<T>) {
        let (tx, rx) = mpsc::channel(channels);
        let handler = Handler { tx };
        let server = Server { queue, rx };
        (server, handler)
    }
}

impl<Q, T> Server<Q, T>
where
    Q: Queue<T>,
{
    pub async fn listen(self, cancel: CancellationToken) -> Result<(), Error> {
        tokio::select! {
            res = self.listening() => match res {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(e)
                },
            },
            _ = cancel.cancelled() => Ok(()),
        }
    }

    async fn listening(mut self) -> Result<(), Error> {
        loop {
            if let Some(recv) = self.rx.recv().await {
                match recv {
                    Command::Enque(item) => {
                        self.queue.enque(item);
                    }
                    Command::Deque(tx) => {
                        let item = self.queue.deque();
                        match tx.send(item) {
                            Ok(_) => {}
                            Err(_) => return Err(Error::ChannelClosed),
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
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
    pub async fn enque(&self, item: T) -> Result<(), Error> {
        self.tx
            .send(Command::Enque(item))
            .await
            .map_err(|_| Error::ChannelClosed)
    }

    pub async fn deque(&self) -> Result<Option<T>, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Command::Deque(tx))
            .await
            .map_err(|_| Error::ChannelClosed)?;
        rx.await.map_err(|_| Error::ChannelClosed)
    }
}
