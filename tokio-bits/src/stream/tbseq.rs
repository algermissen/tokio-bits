use futures::{Stream, Async, Poll};

use ops::inc::Incrementable;
use std::time::{SystemTime, UNIX_EPOCH};

/// A stream that produces a sequence of numbers
/// for a maximum amount of time.
pub struct TBSeqStream<T> {
    next: T,
    start: u64,
    max: u64,
}

impl<T> TBSeqStream<T> {
    pub fn new(initial: T, max_sec: u64) -> Self {
        TBSeqStream {
            next: initial,
            start: Self::ts(),
            max: max_sec,
        }
    }

    fn ts() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .expect("Time went backwards")
    }
}

/// The implementation of the Stream trait uses the generic
/// post_inc() to produce the next value.
impl<T: Incrementable> Stream for TBSeqStream<T> {
    type Item = T;

    type Error = ();
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        if Self::ts() - self.start > self.max {
            Ok(Async::Ready(None))
        } else {
            let v = self.next;
            self.next.post_inc();
            Ok(Async::Ready(Some(v)))
        }
    }
}
