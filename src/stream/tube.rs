use futures::{Stream, Async, Poll, Future};
use futures::Sink;
use futures::StartSend;
use futures::AsyncSink;
use std::time::Duration;
use tokio_core::reactor::Timeout;
use tokio_core::reactor::Handle;
use std::convert::From;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::marker::PhantomData;

/// A Tube is a Stream that can act as its own Sink.
/// Tubes are created using the Tube::pair(...) function which returns
/// the Stream end and the Sink end. Both Stream end and Sink end share
/// the same inner struct.
///
/// Tubes are to be used for periodic work with an arbitrary delay between
/// repititons. The delay is given upon Tube creation.
pub struct Tube<T, E> {
    inner: Rc<RefCell<Inner<T, E>>>,
}

// The Sink end.
pub struct TubeSink<T, E> {
    delay: Duration,
    inner: Weak<RefCell<Inner<T, E>>>,
}

// Inner mutable structure for Stream and Sink ends
struct Inner<T, E> {
    next: Option<T>,
    timeout: Timeout,
    handle: Handle,
    err: PhantomData<E>,
}

impl<T, E> Tube<T, E> {
    /// Create a new Tube and its Sink end and return them as a pair.
    /// The item is the first item to be send down the stream after the
    /// initial delay of dur duration has passed.
    /// Subsequent delays between work will also be this duration.
    /// The delaying future will be spawned on the supplied handle.
    pub fn pair(handle: &Handle, item: T, dur: Duration) -> (Self, TubeSink<T, E>) {
        let timeout = Timeout::new(dur, &handle.clone()).unwrap();
        let stream = Tube {
            inner: Rc::new(RefCell::new(Inner {
                handle: handle.clone(),
                next: Some(item),
                timeout: timeout,
                err: PhantomData,
            })),
        };
        let sink = TubeSink {
            delay: dur,
            inner: Rc::downgrade(&stream.inner),
        };
        (stream, sink)
    }
}

impl<T, E> TubeSink<T, E> {
    /// Supply the next value to inner struct for upcoming work iteration
    fn next(&self, item: T) {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return,
        };
        inner.borrow_mut().next(item, self.delay);
    }
}

impl<T, E> Inner<T, E> {
    // Supply the next item to send down the stream for new work.
    fn next(&mut self, item: T, dur: Duration) {
        self.next = Some(item);
        self.timeout = Timeout::new(dur, &self.handle).unwrap();
    }
}

// E must be convertable from io::Error because this is what the timout
// might yield. It is a bit strange like this; TODO what is a better design?
impl<T, E: ::std::convert::From<::std::io::Error>> Stream for Tube<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let _ = try_ready!(self.inner.borrow_mut().timeout.poll());
        let item = self.inner.borrow_mut().next.take().expect(
            "Stream should not be polled if item is None",
        );
        Ok(Async::Ready(Some(item)))
    }
}

impl<T, E> Sink for TubeSink<T, E> {
    type SinkItem = T;
    type SinkError = E;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.next(item);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}
