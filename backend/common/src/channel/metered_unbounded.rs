use futures::channel::mpsc::{ SendError, TrySendError, UnboundedSender, UnboundedReceiver, unbounded };
use futures::{ Sink, Stream, SinkExt, StreamExt };
use std::sync::atomic::{ AtomicUsize, Ordering };
use std::sync::Arc;
use std::task::Poll;

/// Create an unbounded channel where we record the current length of the message queue.
pub fn metered_unbounded<T>() -> (MeteredUnboundedSender<T>, MeteredUnboundedReceiver<T>) {
    let (tx, rx) = unbounded();
    let len = Arc::new(AtomicUsize::new(0));
    let len2 = Arc::clone(&len);

    let tx = MeteredUnboundedSender {
        inner: tx,
        len: len
    };
    let rx = MeteredUnboundedReceiver {
        inner: rx,
        len: len2
    };

    (tx, rx)
}

/// This is similar to an `UnboundedSender`, except that we keep track
/// of the length of the internal message buffer.
#[derive(Debug, Clone)]
pub struct MeteredUnboundedSender<T> {
    inner: UnboundedSender<T>,
    len: Arc<AtomicUsize>,
}

impl <T> MeteredUnboundedSender<T> {
    /// The current number of messages in the queue.
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Send a message.
    pub fn unbounded_send(&self, item: T) -> Result<(), TrySendError<T>> {
        self.len.fetch_add(1, Ordering::Relaxed);
        self.inner.unbounded_send(item)
    }
}

impl <T> Sink<T> for MeteredUnboundedSender<T> {
    type Error = SendError;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        self.unbounded_send(item).map_err(|e| e.into_send_error())
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx)
    }

    fn poll_close(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx)
    }
}

impl <T> Stream for MeteredUnboundedReceiver<T> {
    type Item = T;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let res = self.inner.poll_next_unpin(cx);
        if matches!(res, Poll::Ready(Some(..))) {
            self.len.fetch_sub(1, Ordering::Relaxed);
        }
        res
    }
}

/// This is similar to an `UnboundedReceiver`, except that we keep track
/// of the length of the internal message buffer.
#[derive(Debug)]
pub struct MeteredUnboundedReceiver<T> {
    inner: UnboundedReceiver<T>,
    len: Arc<AtomicUsize>,
}

impl <T> MeteredUnboundedReceiver<T> {
    /// The current number of messages in the queue.
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn channel_len_consistent_with_msgs() {
        let (tx, mut rx) = metered_unbounded();

        assert_eq!(tx.len(), 0);
        tx.unbounded_send(1).unwrap();
        assert_eq!(tx.len(), 1);
        tx.unbounded_send(2).unwrap();
        assert_eq!(tx.len(), 2);
        tx.unbounded_send(3).unwrap();
        assert_eq!(tx.len(), 3);

        rx.next().await.unwrap();
        assert_eq!(tx.len(), 2);
        rx.next().await.unwrap();
        assert_eq!(tx.len(), 1);
        rx.next().await.unwrap();
        assert_eq!(tx.len(), 0);
    }

    #[tokio::test]
    async fn channel_len_consistent_with_msgs_sink() {
        let (mut tx, mut rx) = metered_unbounded::<usize>();

        assert_eq!(tx.len(), 0);
        tx.send(1).await.unwrap();
        assert_eq!(tx.len(), 1);
        tx.send(2).await.unwrap();
        assert_eq!(tx.len(), 2);
        tx.send(3).await.unwrap();
        assert_eq!(tx.len(), 3);

        rx.next().await.unwrap();
        assert_eq!(tx.len(), 2);
        rx.next().await.unwrap();
        assert_eq!(tx.len(), 1);
        rx.next().await.unwrap();
        assert_eq!(tx.len(), 0);
    }

    #[tokio::test]
    async fn channel_len_consistent_when_send_parallelised() {
        let (mut tx, mut rx) = metered_unbounded::<usize>();

        // Send lots of messages on a bunch of real threads:
        let mut join_handles = vec![];
        for _ in 0..50 {
            let tx = tx.clone();
            let join_handle = std::thread::spawn(move || {
                for i in 0..10000 {
                    tx.unbounded_send(i).unwrap();
                }
            });
            join_handles.push(join_handle);
        }

        // When they are done, our len should be accurate:
        for handle in join_handles {
            handle.join().unwrap();
        }
        assert_eq!(tx.len(), 50 * 10_000);

    }

    #[tokio::test]
    async fn channel_len_consistent_when_send_and_recv_parallelised() {
        let (mut tx, mut rx) = metered_unbounded::<usize>();

        // Send lots of messages on a bunch of real threads:
        let mut join_handles = vec![];
        for _ in 0..50 {
            let tx = tx.clone();
            let join_handle = std::thread::spawn(move || {
                for i in 0..10000 {
                    tx.unbounded_send(i).unwrap();
                }
            });
            join_handles.push(join_handle);
        }

        // While this is happenening, we are trying to receive that same number of msgs:
        for _ in 0..500_000 {
            rx.next().await.unwrap();
        }

        // When they are done, our len should be accurate:
        for handle in join_handles {
            handle.join().unwrap();
        }
        assert_eq!(tx.len(), 0);

    }

}