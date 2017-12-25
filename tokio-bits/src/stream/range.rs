use futures::{Stream, Async, Poll};

use ops::inc::Incrementable;

/// A stream that produces a range of numbers
pub struct RangeStream<T> {
    next: T,
    end: T,
}

impl<T> RangeStream<T> {
    /// Create a new RangeStream producing values starting at start
    /// and ending at one before end.
    pub fn new(start: T, end: T) -> RangeStream<T> {
        RangeStream {
            next: start,
            end: end,
        }
    }
}

/// The implementation of the Stream trait uses the generic
/// post_inc() to produce the next value.
impl<T: Incrementable + ::std::cmp::PartialOrd> Stream for RangeStream<T> {
    type Item = T;

    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if self.next >= self.end {
            Ok(Async::Ready(None))
        } else {
            let v = self.next;
            self.next.post_inc();
            Ok(Async::Ready(Some(v)))
        }
    }
}
