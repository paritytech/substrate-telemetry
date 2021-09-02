use flume::Receiver;
use futures::stream::poll_fn;
use futures::{FutureExt, Stream};
use std::pin::Pin;

/// A drop-in replacement for `flume::RecvStream` to use until a leak is resolved.
pub type FlumeRecvStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

/// This is temporary until `flume`'s `.into_stream()` method no longer leaks.
/// The workaround here is that instead of holding onto a RecvFut, we create one, poll
/// it, and then let it be dropped each time (the same as if we used `recv_async` directly).
/// This is a drop-in replacement for `into_stream` so it'll be easy to switch back.
pub fn flume_receiver_into_stream<'a, T: Send + 'a>(r: Receiver<T>) -> FlumeRecvStream<'a, T> {
    let stream = poll_fn(move |cx| r.recv_async().poll_unpin(cx).map(|r| r.ok()));
    Box::pin(stream)
}
