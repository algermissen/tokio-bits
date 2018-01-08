extern crate hyper;
extern crate futures;
extern crate tokio_core;

use std::io;
use hyper::Client;
use futures::Future;
use futures::Async;
use tokio_core::reactor::{Core, Handle, Timeout};
use std::rc::Rc;

type MyFuture = Future<Item = String, Error = io::Error>;

pub struct UserAgent {
    inner: Rc<Inner>,
}

impl UserAgent {
    pub fn new(handle: &Handle) -> Self {
        UserAgent {
            inner: Rc::new(Inner {
                handle: handle.clone(),
                client: Client::new(&handle),
            }),
        }
    }
}

struct Inner {
    handle: Handle,
    client: Client<hyper::client::HttpConnector, hyper::Body>,
}


struct FsmFuture {
    inner: Rc<Inner>,
    state: State,
}


enum State {
    Requesting(Box<MyFuture>),
    Sleeping(Timeout),
}


impl FsmFuture {
    fn new(inner: &Rc<Inner>) -> Self {
        let f = Self::make_request_future(inner);

        FsmFuture {
            inner: inner.clone(),
            state: State::Requesting(f),
        }
    }

    fn make_timeout(inner: &Inner) -> Timeout {
        Timeout::new(std::time::Duration::new(5, 0), &inner.handle).unwrap()
    }


    fn make_request_future(inner: &Inner) -> Box<MyFuture> {
        let uri = "http://httpbin.org/ip".parse().expect("valid URI");
        let f = inner
            .client
            .get(uri)
            .map(|res| {
                let b = format!("Response: {}", res.status());
                println!("{}", b);
                b
            })
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        Box::new(f)
    }
}

impl Future for FsmFuture {
    type Item = String;
    type Error = io::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        println!("__________ POLLING");
        loop {
            let s = match self.state {
                State::Requesting(ref mut f) => {
                    match f.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(_v)) => {
                            println!("OK");
                            State::Sleeping(FsmFuture::make_timeout(&self.inner))
                        }
                        Err(e) => {
                            println!("Err {:?}", e);
                            State::Sleeping(FsmFuture::make_timeout(&self.inner))
                        }
                    }
                }
                State::Sleeping(ref mut f) => {
                    match f.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(_v)) => {
                            println!("Timer done");
                            let f = FsmFuture::make_request_future(&self.inner);
                            State::Requesting(f)
                        }
                        Err(_e) => {
                            panic!("timer error");
                        }
                    }
                }
            };
            self.state = s;
        }
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let ua = UserAgent::new(&core.handle());
    let f = FsmFuture::new(&ua.inner);

    core.run(f).unwrap();
}
