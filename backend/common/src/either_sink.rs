use futures::sink::Sink;
use pin_project_lite::pin_project;

pin_project! {
    #[project = EitherSinkInner]
    pub enum EitherSink<A, B> {
        A { #[pin] inner: A },
        B { #[pin] inner: B }
    }
}

/// A simple enum that delegates implementation to one of
/// the two possible sinks contained within.
impl<A, B> EitherSink<A, B> {
    pub fn a(val: A) -> Self {
        EitherSink::A { inner: val }
    }
    pub fn b(val: B) -> Self {
        EitherSink::B { inner: val }
    }
}

impl<Item, Error, A, B> Sink<Item> for EitherSink<A, B>
where
    A: Sink<Item, Error = Error>,
    B: Sink<Item, Error = Error>,
{
    type Error = Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            EitherSinkInner::A { inner } => inner.poll_ready(cx),
            EitherSinkInner::B { inner } => inner.poll_ready(cx),
        }
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
        match self.project() {
            EitherSinkInner::A { inner } => inner.start_send(item),
            EitherSinkInner::B { inner } => inner.start_send(item),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            EitherSinkInner::A { inner } => inner.poll_flush(cx),
            EitherSinkInner::B { inner } => inner.poll_flush(cx),
        }
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self.project() {
            EitherSinkInner::A { inner } => inner.poll_close(cx),
            EitherSinkInner::B { inner } => inner.poll_close(cx),
        }
    }
}
