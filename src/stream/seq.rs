use futures::{Stream, Async, Poll};

use ops::inc::Incrementable;

/// A stream that produces a sequence of numbers
pub struct SeqStream<T> {
    next: T,
}

impl<T> SeqStream<T> {
    pub fn new(start: T) -> SeqStream<T> {
        SeqStream { next: start }
    }
}

/// The implementation of the Stream trait uses the generic
/// post_inc() to produce the next value.
impl<T: Incrementable> Stream for SeqStream<T> {
    type Item = T;

    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let v = self.next;
        self.next.post_inc();
        Ok(Async::Ready(Some(v)))
    }
}
