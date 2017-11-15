use futures::{Stream, Async, Poll, Future};

use ops::inc::Incrementable;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::Sink;
use futures::StartSend;
use futures::AsyncSink;

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


/// A stream that produces a sequence of numbers
/// for a maximum amount of time.
#[derive(Clone)]
pub struct ResultStream<T: Clone> {
    next: Option<(T, u64)>,
}

impl<T: Clone> ResultStream<T> {
    pub fn pair(item: T, in_secs: u64) -> (Self, Self) {
        let s = ResultStream { next: Some((item, Self::now() + in_secs)) };
        let t = s.clone();
        (s, t)
    }

    fn next(&mut self, item: T, in_secs: u64) {
        self.next = Some((item, Self::now() + in_secs));
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .expect("Time went backwards")
    }
}

impl<T: Clone> Stream for ResultStream<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        println!("POLL STREAM");
        match self.next {    
            Some((ref item, at)) => {
                println!("POLL STREAM 1");
                if Self::now() >= at {
                    println!("POLL STREAM 2");
                    Ok(Async::Ready(Some(item.clone())))
                } else {
                    println!("POLL STREAM 3");
                    Ok(Async::NotReady)
                }
            }
            None => {
                println!("POLL STREAM 4");
                Ok(Async::NotReady)
            }
        }
    }
}

impl<T: Clone> Sink for ResultStream<T> {
    type SinkItem = T;
    type SinkError = Error;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        println!("START SEND");
        self.next(item, 1);
        Ok(AsyncSink::Ready)
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }

    fn close(&mut self) -> Poll<(), Self::SinkError> {
        Ok(Async::Ready(()))
    }
}




#[cfg(test)]
mod tests {
    extern crate tokio_core;
    use self::tokio_core::reactor::Core;
    use super::*;
    use futures::stream::Stream;

    #[test]

    fn it_works() {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let i: i32 = 1;
        let (source, sink) = ResultStream::pair(i, 1);

        let s = source
            .map(|x| {
                println!("___ X: {:?}", x);
                x
            })
            .map_err(|e| Error::new(ErrorKind::Other, e));

        let x = sink.send_all(s);
        handle.spawn(x.map(|_| ()).map_err(|_| ()));
        core.run(::futures::empty::<(), ()>()).unwrap();


    }

}
