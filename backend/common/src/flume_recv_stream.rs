//! A sort-of drop-in replacement to create a Stream from a flume Receiver, because `flume::Receiver::into_stream()`
//! leaks memory. See:
//!
//! https://github.com/zesterer/flume/issues/88
//!
//! Hopefully we won't need to use these for long; the issue will probably be resolved fairly prompty and we can
//! revert back to using the built-in flume methods.
//!
use flume::{Receiver, r#async::RecvFut};
use futures::{FutureExt, Stream};
use std::pin::Pin;

// /// A drop-in replacement which is similar to `flume::RecvStream`.
// pub type FlumeRecvStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

pub struct FlumeRecvStream<T: 'static>(&'static Receiver<T>, &'static mut Option<RecvFut<'static, T>>);

/// A drop-in replacement for `flume`'s `Receiver::into_stream()` method.
/// Note: This leaks a stream.
pub fn flume_receiver_into_stream<T: Send + 'static>(r: Receiver<T>) -> FlumeRecvStream<T> {
    // Leak the receiver so it's guaranteed to be in one place:
    let r = Box::new(r);
    let r: &'static Receiver<T> = Box::leak(r);

    // Kead the recv_fut holder so it too is in one place:
    let recv_fut = Box::new(None);
    let recv_fut: &'static mut Option<RecvFut<'static, T>> = Box::leak(recv_fut);

    FlumeRecvStream(r, recv_fut)
}

impl <T> Stream for FlumeRecvStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let r: &'static Receiver<T> = self.0;

        // Keep a RecvFut around until it's ready
        if self.1.is_none() {
            *self.1 = Some(r.recv_async());
        }

        // Poll it:
        let recv_fut = self.1.as_mut().expect("Always something stored here");
        let res = recv_fut.poll_unpin(cx).map(|r| r.ok());

        // Once it's ready, drop it to prevent leaks.
        if res.is_ready() {
            *self.1 = None;
        }

        // Return the result:
        res
    }
}

impl <T> Drop for FlumeRecvStream<T> {
    fn drop(&mut self) {
        // Drop the RecvFut, which depends on Receiver, first.
        drop(unsafe { Box::from_raw(self.1) });
        // Then drop the Receiver
        drop(unsafe { Box::from_raw(self.0 as *const _ as *mut Receiver<T>) });
    }
}