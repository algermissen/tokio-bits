extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate rand;

use std::io;
use hyper::Client;
use futures::Future;
use futures::Async;
use tokio_core::reactor::{Core, Handle, Timeout};
use std::rc::Rc;

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


struct FsmFuture<T, E> {
    inner: Rc<Inner>,
    state: State<T, E>,
    make: Box<Fn(&Inner) -> Box<Future<Item = T, Error = E>>>,
}


enum State<T, E> {
    Requesting(Box<Future<Item = T, Error = E>>, i32),
    Sleeping(Timeout, i32),
}


impl<T, E> FsmFuture<T, E> {
    fn new<F>(inner: &Rc<Inner>, make: Box<F>) -> Self
    where
        F: 'static + Fn(&Inner) -> Box<Future<Item = T, Error = E>>,
    {

        let f = make(inner);
        FsmFuture {
            inner: inner.clone(),
            state: State::Requesting(f, 1),
            make: make,
        }
    }
}

/// Implements 'Full Jitter' as described in
/// https://aws.amazon.com/de/blogs/architecture/exponential-backoff-and-jitter/
fn timeout<T: rand::Rng>(rng: &mut T, handle: &Handle, attempt: i32) -> Result<Timeout, io::Error> {
    let v: u64 = ::std::cmp::min(1 * (2 as i32).pow(attempt as u32), 60) as u64;
    let nsec = rng.gen_range(0u64, v);
    println!("BO: {}", nsec);
    Timeout::new(::std::time::Duration::new(nsec, 0), handle)
}

impl<T, E> Future for FsmFuture<T, E> {
    type Item = T;
    type Error = E;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        println!("__________ POLLING");
        loop {
            self.state = match self.state {
                State::Requesting(ref mut f, count) => {
                    match f.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(v)) => return Ok(Async::Ready(v)),
                        Err(e) => {
                            let count = count + 1;
                            //println!("Err {}", e);
                            let t = timeout(&mut rand::thread_rng(), &self.inner.handle, count)
                                .unwrap();
                            State::Sleeping(t, count)
                        }
                    }
                }
                State::Sleeping(ref mut f, count) => {
                    match f.poll() {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(_)) => {
                            println!("Timer done");
                            let f = (self.make)(&self.inner);
                            State::Requesting(f, count)
                        }
                        Err(e) => panic!("TODO"), //return Err(From::from(e)),
                    }
                }
            };
        }
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let ua = UserAgent::new(&core.handle());

    let f = |inner: &Inner| -> Box<Future<Item = String, Error = io::Error>> {
        let uri = "http://xxhttpbin.org/ip".parse().expect("valid URI");
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
    };

    let f = FsmFuture::new(&ua.inner, Box::new(f));

    core.run(f).unwrap();
}
