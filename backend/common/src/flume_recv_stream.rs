//! A sort-of drop-in replacement to create a Stream from a flume Receiver, because `flume::Receiver::into_stream()`
//! leaks memory. See:
//!
//! https://github.com/zesterer/flume/issues/88
//!
//! Hopefully we won't need to use these for long; the issue will probably be resolved fairly prompty and we can
//! revert back to using the built-in flume methods.
//!
use flume::Receiver;
use futures::stream::poll_fn;
use futures::{FutureExt, Stream};
use std::pin::Pin;

/// A drop-in replacement which is similar to `flume::RecvStream`.
pub type FlumeRecvStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

/// A drop-in replacement for `flume`'s `Receiver::into_stream()` method.
pub fn flume_receiver_into_stream<'a, T: Send + 'a>(r: Receiver<T>) -> FlumeRecvStream<'a, T> {
    let stream = poll_fn(move |cx| r.recv_async().poll_unpin(cx).map(|r| r.ok()));
    Box::pin(stream)
}
