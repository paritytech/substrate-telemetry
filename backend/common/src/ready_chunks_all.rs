//! [`futures::StreamExt::ready_chunks()`] internally stores a vec with a certain capacity, and will buffer up
//! up to that many items that are ready from the underlying stream before returning either when we run out of
//! Poll::Ready items, or we hit the capacity.
//!
//! This variation has no fixed capacity, and will buffer everything it can up at each point to return. This is
//! better when the amount of items varies a bunch (and we don't want to allocate a fixed capacity every time),
//! and can help ensure that we process as many items as possible each time (rather than only up to capacity items).
//!
//! Code is adapted from the futures implementation
//! (see [ready_chunks.rs](https://docs.rs/futures-util/0.3.15/src/futures_util/stream/stream/ready_chunks.rs.html)).

use futures::stream::Fuse;
use futures::StreamExt;
use core::mem;
use core::pin::Pin;
use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};
use pin_project_lite::pin_project;

pin_project! {
    /// Buffer up all Ready items in the underlying stream each time
    /// we attempt to retrieve items from it, and return a Vec of those
    /// items.
    #[derive(Debug)]
    #[must_use = "streams do nothing unless polled"]
    pub struct ReadyChunksAll<St: Stream> {
        #[pin]
        stream: Fuse<St>,
        items: Vec<St::Item>,
    }
}

impl<St: Stream> ReadyChunksAll<St>
where
    St: Stream,
{
    pub fn new(stream: St) -> Self {
        Self {
            stream: stream.fuse(),
            items: Vec::new()
        }
    }
}

impl<St: Stream> Stream for ReadyChunksAll<St> {
    type Item = Vec<St::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            match this.stream.as_mut().poll_next(cx) {
                // Flush all collected data if underlying stream doesn't contain
                // more ready values
                Poll::Pending => {
                    return if this.items.is_empty() {
                        Poll::Pending
                    } else {
                        Poll::Ready(Some(mem::replace(this.items, Vec::new())))
                    }
                }

                // Push the ready item into the buffer
                Poll::Ready(Some(item)) => {
                    this.items.push(item);
                }

                // Since the underlying stream ran out of values, return what we
                // have buffered, if we have anything.
                Poll::Ready(None) => {
                    let last = if this.items.is_empty() {
                        None
                    } else {
                        let full_buf = mem::replace(this.items, Vec::new());
                        Some(full_buf)
                    };

                    return Poll::Ready(last);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Look at the underlying stream's size_hint. If we've
        // buffered some items, we'll return at least that Vec,
        // giving us a lower bound 1 greater than the underlying.
        // The upper bound is, worst case, our vec + each individual
        // item in the underlying stream.
        let chunk_len = if self.items.is_empty() { 0 } else { 1 };
        let (lower, upper) = self.stream.size_hint();
        let lower = lower.saturating_add(chunk_len);
        let upper = match upper {
            Some(x) => x.checked_add(chunk_len),
            None => None,
        };
        (lower, upper)
    }
}

impl<St: FusedStream> FusedStream for ReadyChunksAll<St> {
    fn is_terminated(&self) -> bool {
        self.stream.is_terminated() && self.items.is_empty()
    }
}
