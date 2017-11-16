use futures::{Stream, Async, Poll, Future};
use ops::inc::Incrementable;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::Sink;
use futures::sink::SendAll;
use futures::StartSend;
use futures::AsyncSink;
use std::time::Duration;
use tokio_core::reactor::Timeout;
use tokio_core::reactor::Handle;
use std::convert::From;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    error: Box<::std::error::Error + Send + Sync>,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::Other => "other",
        }
    }
}


impl ::std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.kind.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    /// Other error with yet unspecified meaning.
    Other,
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        self.error.description()
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        self.error.cause()
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<::std::error::Error + Send + Sync>>,
    {
        Error {
            kind: kind,
            error: error.into(),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Error::new(ErrorKind::Other, e)
    }
}

pub struct ResultStream<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

struct Inner<T> {
    next: Option<T>,
    timeout: Timeout,
    handle: Handle,
}

pub struct MySink<T> {
    inner: Weak<RefCell<Inner<T>>>,
}



impl<T> ResultStream<T> {
    pub fn pair(handle: &Handle, item: T, dur: Duration) -> (Self, MySink<T>) {
        let handle = handle.clone();
        let handle2 = handle.clone();
        let timeout = Timeout::new(dur, &handle2).unwrap();

        let stream = ResultStream {
            inner: Rc::new(RefCell::new(Inner {
                handle: handle2,
                next: Some(item),
                timeout: timeout,
            })),
        };
        let sink = MySink { inner: Rc::downgrade(&stream.inner) };
        (stream, sink)
    }
}

impl<T> Inner<T> {
    fn next(&mut self, item: T, dur: Duration) {
        self.next = Some(item);
        self.timeout = Timeout::new(dur, &self.handle).unwrap();
    }
}

impl<T> MySink<T> {
    fn next(&self, item: T, dur: Duration) {
        let inner = match self.inner.upgrade() {
            Some(inner) => inner,
            None => return,
        };
        inner.borrow_mut().next(item, dur);
    }
}

impl<T> Stream for ResultStream<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let _ = try_ready!(self.inner.borrow_mut().timeout.poll());
        let item = self.inner.borrow_mut().next.take().unwrap();
        Ok(Async::Ready(Some(item)))
    }
}

impl<T> Sink for MySink<T> {
    type SinkItem = T;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        //println!("START SEND {}", item);
        println!("START SEND");
        let dur = Duration::from_millis(10000);
        self.next(item, dur);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}
