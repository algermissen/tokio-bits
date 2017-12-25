
extern crate futures;
extern crate tokio_core;
extern crate hyper;
extern crate tokio_bits;

use futures::Future;
use futures::Stream;
use futures::Sink;

use std::time::Duration;
use std::convert::From;

use tokio_core::reactor::Core;
use hyper::client::Client;

use tokio_bits::stream::tube::*;

struct Err(i32);

impl From<::std::io::Error> for Err {
    fn from(_: ::std::io::Error) -> Self {
        Err(1)
    }
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let dur = Duration::from_millis(10000);
    let (source, sink) = Tube::pair(&handle, 0, dur);

    let client = ::std::rc::Rc::new(Client::new(&handle));

    let s = source
        .map(|x| {
            println!("Item: {:?}", x);
            // Let's change the item to have some 'progress'.
            x + 1
        })
        .and_then(move |x| {
            // Do async work to show the intended use of Tube.
            let uri: ::hyper::Uri = "http://httpbin.org/ip".parse().unwrap();
            let work = client
                .get(uri)
                .map(move |res| {
                    println!("Response Status: {}", res.status());
                    x
                })
                .map_err(|_| Err(1));
            work
        });

    let a = sink.send_all(s);
    handle.spawn(a.map(|_| ()).map_err(|_| ()));
    core.run(::futures::empty::<(), ()>()).unwrap();


}
