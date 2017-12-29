//! This crate is a collection of modules useful for working
//! with Tokio when spawning off child processes.
extern crate tokio_io;
extern crate tokio_process;
extern crate futures;
extern crate tokio_core;

use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_io::io::lines;
use futures::Stream;
use std::io::{BufReader, ErrorKind, Error};
use std::process::Stdio;

/// Hdl is a handler type combining the child future and
/// stdout and stderr streams returned from the execute
/// function.
pub struct Hdl {
    /// The child future. If this goes out of scope, the process will be killed.
    /// Use the forget() method to avoid that.
    pub child: ::tokio_process::Child,
    /// Line based stream of child process' stdout
    pub stdout: Box<Stream<Item = String, Error = Error>>,
    /// Line based stream of child process' stderr
    pub stderr: Box<Stream<Item = String, Error = Error>>,
}

// Create a new command from the given command and arguments and
// spawn a child that runs the given command and returns line-by-line
// streams of its stdout and stderr.
pub fn execute<'a, I>(handle: &Handle, c: &str, args: I) -> Result<Hdl, Error>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut cmd = ::std::process::Command::new(c);
    cmd.args(args);

    // We want to read stderr and stdout
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Spawn the child process
    let mut child = try!(cmd.spawn_async(handle));

    // Create line-based streams from stdout and stderr of child
    let stdout_stream = match child.stdout().take() {
        Some(reader) => Box::new(lines(BufReader::new(reader))),
        None => return Result::Err(Error::new(ErrorKind::Other, "stdout ws not captured")),
    };
    let stderr_stream = match child.stderr().take() {
        Some(reader) => Box::new(lines(BufReader::new(reader))),
        None => return Result::Err(Error::new(ErrorKind::Other, "stderr ws not captured")),
    };

    let h = Hdl {
        child: child,
        stdout: stdout_stream,
        stderr: stderr_stream,
    };
    Result::Ok(h)
}
