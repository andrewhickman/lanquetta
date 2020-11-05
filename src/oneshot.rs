use std::future::Future;

use futures::future::{select, Either};
use tokio::sync::watch;

pub fn new<T, F>(fut: F) -> Receiver<T>
where
    T: Clone + Send + Sync + 'static,
    F: Future<Output = T> + Send + 'static,
{
    let (mut sender, receiver) = watch::channel(None);

    tokio::spawn(async move {
        let value = {
            let closed = sender.closed();
            futures::pin_mut!(fut);
            futures::pin_mut!(closed);
            match select(fut, closed).await {
                Either::Left((value, _)) => value,
                Either::Right(_) => return,
            }
        };

        let _ = sender.broadcast(Some(value));
    });

    Receiver { watch: receiver }
}

pub struct Receiver<T> {
    watch: watch::Receiver<Option<T>>,
}

impl<T: Clone> Receiver<T> {
    pub async fn recv(&mut self) -> T {
        let value = self.watch.recv().await.expect("sender did not complete");
        match value {
            Some(value) => value,
            None => self
                .watch
                .recv()
                .await
                .expect("sender did not complete")
                .expect("sender sent None"),
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            watch: self.watch.clone(),
        }
    }
}
