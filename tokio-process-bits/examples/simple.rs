extern crate tokio_process_bits;
extern crate futures;
extern crate tokio_core;

use tokio_process_bits::execute;
use futures::Stream;

pub fn main() {
    let mut core = ::tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let c1 = execute(&handle, "ping", vec!["127.0.0.1"]).unwrap();
    let c2 = execute(&handle, "ping", vec!["0.0.0.0"]).unwrap();

    let lines = c1.stdout.select(c2.stdout).for_each(|line| {
        println!("LINE: {}", line);
        ::futures::future::ok(())
    });

    core.run(lines).unwrap();
}
